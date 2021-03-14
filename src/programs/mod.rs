use nannou::prelude::*;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};
use std::time;

use crate::util;

mod config;
pub mod program;
mod shaders;
pub mod uniforms;

/// Stores GPU programs and related data.
/// Manages the maintenance of shader programs.
/// - listens to directory
/// - compiles code
/// - manages modules
/// - handles errors
/// - builds render pipelines
/// - manages uniform buffers
#[allow(dead_code)] // needed for shader_watcher
pub struct ProgramStore {
    pub buffer_store: uniforms::BufferStore,
    pub current_subscriptions: Option<uniforms::UniformSubscriptions>,
    pub error: Option<String>,
    pub folder_index: usize,
    pub folder_names: Option<Vec<String>>,
    pub program_names: Option<Vec<String>>,
    pub program_index: usize,

    changes_channel: Receiver<DebouncedEvent>,
    config: Option<config::Config>,
    current_program: Option<program::Program>,
    #[cfg(target_os = "macos")]
    shader_watcher: notify::FsEventWatcher,
    #[cfg(target_os = "linux")]
    shader_watcher: notify::INotifyWatcher,
    #[cfg(target_os = "windows")]
    shader_watcher: notify::ReadDirectoryChangesWatcher,
}

impl ProgramStore {
    pub fn new(app: &App, device: &wgpu::Device, size: Vector2) -> Self {
        let buffer_store = uniforms::BufferStore::new(device, size);

        // setup shader watcher
        let (send_channel, changes_channel) = channel();
        let mut shader_watcher = watcher(send_channel, time::Duration::from_secs(1)).unwrap();
        let shader_path = util::shaders_path_string(app);
        shader_watcher
            .watch(shader_path.as_str(), RecursiveMode::Recursive)
            .unwrap();

        Self {
            buffer_store,
            changes_channel,
            config: None,
            current_program: None,
            current_subscriptions: None,
            error: None,
            folder_index: 0,
            folder_names: None,
            program_index: 0,
            program_names: None,
            shader_watcher,
        }
    }

    fn get_folder_name(&self) -> Option<String> {
        let folder_names = &self.folder_names.as_ref()?;
        Some(folder_names[self.folder_index.clone()].clone())
    }

    fn get_program_name(&self) -> Option<String> {
        let program_names = &self.program_names.as_ref()?;
        Some(program_names[self.program_index.clone()].clone())
    }

    fn get_default_folder_index(
        &self,
        folder_names: &Vec<String>,
        name: &String,
    ) -> Result<usize, String> {
        match folder_names.iter().position(|n| *n == *name) {
            Some(i) => Ok(i),
            None => Err(format!("Invalid default folder '{}'", name)),
        }
    }

    fn get_default_program_index(
        &self,
        program_names: &Vec<String>,
        name: &String,
    ) -> Result<usize, String> {
        match program_names.iter().position(|n| *n == *name) {
            Some(i) => Ok(i),
            None => Err(format!("Invalid default program '{}'", name)),
        }
    }

    /// Compile current program with latest shader code.
    /// Call once after initialization.
    fn compile_current(&mut self, app: &App, device: &wgpu::Device, num_samples: u32) {
        let current_program = match &mut self.current_program {
            Some(p) => p,
            None => {
                return;
            }
        };

        // update the current GPU program to use the latest code
        let buffers = &self.buffer_store.buffers;
        // map the current program's uniform list to a list of bind group layouts
        let bind_group_layouts = &current_program
            .config
            .uniforms
            .iter()
            .map(|u| &buffers.get(&u.to_string()).unwrap().bind_group_layout)
            .collect::<Vec<&wgpu::BindGroupLayout>>()[..];

        // update the program with the new shader code and appropriate layout description
        let layout_desc = wgpu::PipelineLayoutDescriptor { bind_group_layouts };
        current_program.compile(app, device, &layout_desc, num_samples);
    }

    /// Read fresh config and recompile
    pub fn configure(&mut self, app: &App, device: &wgpu::Device, num_samples: u32) {
        // first, clear the current program
        if let Some(current_program) = &mut self.current_program {
            current_program.clear();
        }

        let config = match config::get_config(app) {
            Ok(c) => c,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };

        self.config = Some(config.clone());
        let folder_names = config.get_folder_names();

        let old_folder_name_opt = self.get_folder_name();

        self.folder_names = Some(folder_names.clone());
        let new_folder_name_opt = self.get_folder_name();

        let mut folder_name_opt: Option<String> = None;
        let mut using_defaults = false;

        // if the name matches with new config and old then use the old folder name
        if old_folder_name_opt.is_some() && new_folder_name_opt.is_some() {
            let old_folder_name = old_folder_name_opt.unwrap();
            if old_folder_name == new_folder_name_opt.unwrap() {
                folder_name_opt = Some(old_folder_name);
            }
        }

        // if that didn't work read the default folder
        if folder_name_opt.is_none() {
            let folder_index = match self.get_default_folder_index(&folder_names, &config.default) {
                Ok(i) => i,
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            };
            self.folder_index = folder_index;
            folder_name_opt = Some(folder_names[folder_index].clone());
            using_defaults = true;
        }

        let mut folder_name = folder_name_opt.unwrap();

        let missing_folder = format!("Missing default folder config '{}'", config.default);

        // read the folder config, try the default folder on failure
        let folder_config = match config.folders.get(&folder_name) {
            Some(c) => c,
            None => {
                // we already tried the new config, abort
                if using_defaults {
                    self.error = Some(missing_folder);
                    return;
                }

                // maybe the config updated, get the new default index
                let folder_index =
                    match self.get_default_folder_index(&folder_names, &config.default) {
                        Ok(i) => i,
                        Err(e) => {
                            self.error = Some(e);
                            return;
                        }
                    };

                self.folder_index = folder_index;
                folder_name = folder_names[folder_index].clone();
                using_defaults = true;

                // retry
                match config.folders.get(&folder_name) {
                    Some(c) => c,
                    None => {
                        self.error = Some(missing_folder);
                        return;
                    }
                }
            }
        };

        let program_names = folder_config.get_program_names();
        let mut program_name_opt: Option<String> = None;

        // try to use the old program if it is still valid
        if !using_defaults {
            let old_program_name_opt = self.get_program_name();
            self.program_names = Some(program_names.clone());
            let new_program_name_opt = self.get_program_name();

            if old_program_name_opt.is_some() && new_program_name_opt.is_some() {
                let old_program_name = old_program_name_opt.unwrap();
                if old_program_name == new_program_name_opt.unwrap() {
                    program_name_opt = Some(old_program_name);
                }
            }
        } else {
            self.program_names = Some(program_names.clone());
        }

        // fallback on the default program
        if program_name_opt.is_none() {
            let program_index = match self
                .get_default_program_index(&program_names.clone(), &folder_config.default)
            {
                Ok(i) => i,
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            };

            self.program_index = program_index;
            program_name_opt = Some(folder_config.default.clone());
            using_defaults = true;
        }

        let mut program_name = program_name_opt.unwrap();
        let missing_program = format!("Missing default program config '{}'", folder_config.default);

        // read the program config, falling back on the default on failure
        let program_config = match folder_config.programs.get(&program_name) {
            Some(c) => c,
            None => {
                // already using defaults, abort
                if using_defaults {
                    self.error = Some(missing_program);
                    return;
                }

                // maybe the config updated, get the new default index
                let program_index = match self
                    .get_default_program_index(&program_names.clone(), &folder_config.default)
                {
                    Ok(i) => i,
                    Err(e) => {
                        self.error = Some(e);
                        return;
                    }
                };

                self.program_index = program_index;
                program_name = program_names[program_index].clone();

                // retry
                match folder_config.programs.get(&program_name) {
                    Some(c) => c,
                    None => {
                        self.error = Some(missing_program);
                        return;
                    }
                }
            }
        };

        // create current program
        let current_program = program::Program::new(program_config.clone(), folder_name.clone());
        self.current_program = Some(current_program);

        // get subscriptions and initialize
        let current_subscriptions = uniforms::get_subscriptions(&program_config.uniforms);
        self.buffer_store.set_program_defaults(
            app,
            device,
            &current_subscriptions,
            &program_config.defaults,
        );

        self.current_subscriptions = Some(current_subscriptions);
        self.compile_current(app, device, num_samples);
        self.error = None;
    }

    /// Check if changes have been made to shaders and recompile if needed.
    /// Call every timestep.
    pub fn update_shaders(&mut self, app: &App, device: &wgpu::Device, num_samples: u32) {
        // check for changes
        if let Ok(event) = self.changes_channel.try_recv() {
            if let DebouncedEvent::Write(path) = event {
                let path_str = path.into_os_string().into_string().unwrap();
                println!("changes written to: {}", path_str);

                if path_str.ends_with(".json") {
                    self.configure(app, device, num_samples);
                } else {
                    self.compile_current(app, device, num_samples);
                }
            }
        }

        if let Some(current_program) = &mut self.current_program {
            if current_program.is_new()
                || self.buffer_store.image_uniforms.updated
                || self.buffer_store.webcam_uniforms.updated
            {
                self.compile_current(app, device, num_samples);

                if self.buffer_store.image_uniforms.updated {
                    self.buffer_store.image_uniforms.updated = false;
                }

                if self.buffer_store.webcam_uniforms.updated {
                    self.buffer_store.webcam_uniforms.updated = false;
                }
            }
        }
    }

    /// Update uniform data.
    /// Call every timestep.
    pub fn update_uniforms(&mut self, device: &wgpu::Device) {
        if let Some(current_subscriptions) = self.current_subscriptions.as_ref() {
            self.buffer_store.update(device, current_subscriptions);
        }
    }

    /// Fetch current GPU program.
    pub fn current_pipeline(&self) -> Option<&wgpu::RenderPipeline> {
        let current_program = &self.current_program.as_ref()?;
        current_program.pipeline.as_ref()
    }

    /// Selects the current program performs any housekeeping / initialization
    pub fn select_program(
        &mut self,
        app: &App,
        device: &wgpu::Device,
        selected: usize,
        force: bool,
    ) -> Option<bool> {
        if self.error.is_none() && !force && selected == self.program_index {
            return None;
        }

        let program_names = &self.program_names.as_ref()?;
        let name = &program_names[selected];

        // first, clear the current program
        if let Some(current_program) = &mut self.current_program {
            current_program.clear();
        }

        // next, update the current program and uniforms
        // it will be compiled in the next update()
        println!("program selected: {}", name);
        self.program_index = selected;

        let folder_name = self.get_folder_name()?;
        let config = &self.config.as_ref()?;
        let folder_config = config.folders.get(&folder_name).unwrap();

        let program_config = match folder_config.programs.get(name) {
            Some(c) => c,
            None => {
                self.error = Some(format!("Missing program config '{}'", name));
                return None;
            }
        };

        self.current_program = Some(program::Program::new(
            program_config.clone(),
            folder_name.clone(),
        ));

        let current_subscriptions = uniforms::get_subscriptions(&program_config.uniforms);
        self.buffer_store.set_program_defaults(
            app,
            device,
            &current_subscriptions,
            &program_config.defaults,
        );

        self.current_subscriptions = Some(current_subscriptions);
        self.error = None;

        return Some(true);
    }

    /// Selects the current shader folder
    pub fn select_folder(
        &mut self,
        app: &App,
        device: &wgpu::Device,
        selected: usize,
    ) -> Option<bool> {
        if self.error.is_none() && selected == self.folder_index {
            return None;
        }

        self.folder_index = selected;
        let folder_names = &self.folder_names.as_ref()?;
        let name = &folder_names[selected];
        let config = &self.config.as_ref()?;
        let folder_config = match config.folders.get(name) {
            Some(c) => c,
            None => {
                self.error = Some(format!("Missing folder config '{}'", name));
                return None;
            }
        };

        let mut program_names = vec![];
        for (name, _) in folder_config.programs.iter() {
            program_names.push(name.clone());
        }
        program_names.sort();

        let program_index = match program_names
            .iter()
            .position(|n| *n == folder_config.default)
        {
            Some(i) => i,
            None => {
                self.error = Some(format!(
                    "Invalid default program '{}'",
                    folder_config.default
                ));
                return None;
            }
        };

        self.program_names = Some(program_names);
        self.select_program(app, device, program_index, true)
    }

    /// Update GPU uniform buffers with current data.
    /// Call in draw() before rendering.
    pub fn update_uniform_buffers(
        &self,
        device: &wgpu::Device,
        encoder: &mut nannou::wgpu::CommandEncoder,
    ) {
        if let Some(current_subscriptions) = self.current_subscriptions.as_ref() {
            self.buffer_store
                .update_buffers(device, encoder, current_subscriptions);
        }
    }

    /// Fetch the appropriate bind groups to set positions for the current program.
    /// Call in draw() right before rendering.
    pub fn get_bind_groups(&self) -> Option<Vec<&wgpu::BindGroup>> {
        let current_program = self.current_program.as_ref()?;
        Some(
            current_program
                .config
                .uniforms
                .iter()
                .map(|u| &self.buffer_store.buffers.get(u).unwrap().bind_group)
                .collect::<Vec<&wgpu::BindGroup>>(),
        )
    }

    pub fn get_program_errors(&self) -> Option<&program::ProgramErrors> {
        let current_program = &self.current_program.as_ref()?;
        Some(&current_program.errors)
    }

    pub fn pause(&mut self) {
        if let Some(current_subscriptions) = &self.current_subscriptions {
            self.buffer_store.pause(current_subscriptions);
        }
    }

    pub fn unpause(&mut self) {
        if let Some(current_subscriptions) = &self.current_subscriptions {
            self.buffer_store.unpause(current_subscriptions);
        }
    }
}
