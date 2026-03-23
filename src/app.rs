// SPDX-License-Identifier: GPL-v3

use cosmic::app::context_drawer;
use cosmic::iced::Length;
use cosmic::iced::Subscription;
use cosmic::widget::{self, icon, nav_bar};
use cosmic::{iced_futures, prelude::*};
use cosmic::iced::futures::SinkExt;
use std::time::Duration;
use sysinfo::{Disks, Networks, System};

pub struct AppModel {
    core: cosmic::Core,
    nav: nav_bar::Model,
    sys: System,
    networks: Networks,
    disks: Disks,
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    gpu_name: String,
    gpu_usage: f32,
    gpu_memory_used: u64,
    gpu_memory_total: u64,
    process_sort: ProcessSort,
    hostname: String,
    username: String,
    os_name: String,
    kernel: String,
    uptime: u64,
    shell: String,
    de_wm: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Cpu,
    Memory,
    Alphabetical,
}

#[derive(Debug, Clone)]
pub enum Message {
    RefreshSystemInfo,
    SortProcesses(ProcessSort),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.zachvlat.cosmic-monitor";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Overview")
            .data::<Page>(Page::Overview)
            .icon(icon::from_name("computer-symbolic"))
            .activate();

        nav.insert()
            .text("CPU")
            .data::<Page>(Page::Cpu)
            .icon(icon::from_name("cpu-symbolic"));

        nav.insert()
            .text("GPU")
            .data::<Page>(Page::Gpu)
            .icon(icon::from_name("video-card-symbolic"));

        nav.insert()
            .text("Memory")
            .data::<Page>(Page::Memory)
            .icon(icon::from_name("drive-harddisk-symbolic"));

        nav.insert()
            .text("Processes")
            .data::<Page>(Page::Processes)
            .icon(icon::from_name("process-working-symbolic"));

        nav.insert()
            .text("Network")
            .data::<Page>(Page::Network)
            .icon(icon::from_name("network-wireless-symbolic"));

        nav.insert()
            .text("Disks")
            .data::<Page>(Page::Disks)
            .icon(icon::from_name("drive-harddisk-symbolic"));

        let mut sys = System::new_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        sys.refresh_all();
        let memory_total = sys.total_memory();

        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let username = std::env::var("USER").unwrap_or_else(|_| "Unknown".to_string());
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| shell.clone());
        let de_wm = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "Unknown".to_string());

        let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let uptime = System::uptime();

        let os_name = Self::get_os_name();

        let mut app = AppModel {
            core,
            nav,
            sys,
            networks,
            disks,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total,
            gpu_name: String::new(),
            gpu_usage: 0.0,
            gpu_memory_used: 0,
            gpu_memory_total: 0,
            process_sort: ProcessSort::Cpu,
            hostname,
            username,
            os_name,
            kernel,
            uptime,
            shell: shell_name,
            de_wm,
        };

        app.refresh_gpu_info();

        let command = app.update_title();
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        None
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Overview => self.overview_view(),
            Page::Cpu => self.cpu_view(),
            Page::Memory => self.memory_view(),
            Page::Processes => self.processes_view(),
            Page::Network => self.network_view(),
            Page::Disks => self.disks_view(),
            Page::Gpu => self.gpu_view(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(|| {
            iced_futures::stream::channel(1, |mut emitter: iced_futures::futures::channel::mpsc::Sender<Message>| async move {
                let mut interval = tokio::time::interval(Duration::from_secs(2));
                loop {
                    interval.tick().await;
                    let _ = emitter.send(Message::RefreshSystemInfo).await;
                }
            })
        })
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::RefreshSystemInfo => {
                self.sys.refresh_all();
                self.networks.refresh();
                self.disks.refresh();
                self.cpu_usage = self.sys.global_cpu_usage();
                self.memory_used = self.sys.used_memory();
                self.memory_total = self.sys.total_memory();
                self.uptime = System::uptime();
                self.refresh_gpu_info();
            }
            Message::SortProcesses(sort) => {
                self.process_sort = sort;
            }
        }
        Task::none()
    }

    fn on_nav_select(
        &mut self,
        id: nav_bar::Id,
    ) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.update_title()
    }
}

impl AppModel {
    fn get_os_name() -> String {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| {
                        let name = line.trim_start_matches("PRETTY_NAME=");
                        name.trim_matches('"').to_string()
                    })
            })
            .unwrap_or_else(|| "Linux".to_string())
    }

    fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let mins = (seconds % 3600) / 60;
        
        match (days, hours, mins) {
            (0, 0, m) => format!("{} min", m),
            (0, h, 0) => format!("{}h", h),
            (0, h, m) => format!("{}h {}m", h, m),
            (d, 0, 0) => format!("{} day{}", d, if d > 1 { "s" } else { "" }),
            (d, h, 0) => format!("{}d {}h", d, h),
            (d, h, m) => format!("{}d {}h {}m", d, h, m),
        }
    }

    fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let title = if let Some(page) = self.nav.text(self.nav.active()) {
            format!("System Monitor — {}", page)
        } else {
            "System Monitor".to_string()
        };

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(title, id)
        } else {
            Task::none()
        }
    }

    fn overview_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 16;

        let cpu_name = self.sys.cpus().first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let memory_used_gb = self.memory_used as f64 / 1_073_741_824.0;
        let memory_total_gb = self.memory_total as f64 / 1_073_741_824.0;
        let memory_percent = if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let mut disk_total: u64 = 0;
        let mut disk_used: u64 = 0;
        for disk in self.disks.iter() {
            disk_total += disk.total_space();
            disk_used += disk.total_space() - disk.available_space();
        }
        let disk_total_gb = disk_total as f64 / 1_073_741_824.0;
        let disk_used_gb = disk_used as f64 / 1_073_741_824.0;
        let disk_percent = if disk_total > 0 {
            (disk_used as f64 / disk_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let uptime_str = Self::format_uptime(self.uptime);

        let label_col = widget::column::with_children(vec![
            widget::text::body("User").into(),
            widget::text::body("Hostname").into(),
            widget::text::body("OS").into(),
            widget::text::body("Kernel").into(),
            widget::text::body("Uptime").into(),
            widget::text::body("Shell").into(),
            widget::text::body("DE/WM").into(),
            widget::text::body("CPU").into(),
            widget::text::body("GPU").into(),
            widget::text::body("Memory").into(),
            widget::text::body("Disk").into(),
        ]).spacing(8);

        let value_col = widget::column::with_children(vec![
            widget::text::body(self.username.clone()).into(),
            widget::text::body(self.hostname.clone()).into(),
            widget::text::body(self.os_name.clone()).into(),
            widget::text::body(self.kernel.clone()).into(),
            widget::text::body(uptime_str.clone()).into(),
            widget::text::body(self.shell.clone()).into(),
            widget::text::body(self.de_wm.clone()).into(),
            widget::text::body(cpu_name.clone()).into(),
            widget::text::body(self.gpu_name.clone()).into(),
            widget::text::body(format!("{:.1} / {:.1} GB ({:.0}%)", memory_used_gb, memory_total_gb, memory_percent)).into(),
            widget::text::body(format!("{:.0} / {:.0} GB ({:.0}%)", disk_used_gb, disk_total_gb, disk_percent)).into(),
        ]).spacing(8);

        let info_row = widget::row::with_capacity(2)
            .spacing(24)
            .push(
                widget::container(label_col)
                    .padding([0, 16, 0, 0])
                    .align_y(cosmic::iced::Alignment::Start)
            )
            .push(
                widget::container(value_col)
                    .padding([0, 0, 0, 0])
                    .align_y(cosmic::iced::Alignment::Start)
            );

        let usage_section = cosmic::widget::settings::section()
            .title("Usage")
            .add(
                cosmic::widget::settings::item::builder("CPU")
                    .control(
                        widget::row::with_children(vec![
                            widget::progress_bar(0.0..=100.0, self.cpu_usage).into(),
                            widget::text::body(format!(" {:.0}%", self.cpu_usage)).into(),
                        ]).spacing(8)
                    ),
            )
            .add(
                cosmic::widget::settings::item::builder("GPU")
                    .control(
                        widget::row::with_children(vec![
                            widget::progress_bar(0.0..=100.0, self.gpu_usage).into(),
                            widget::text::body(format!(" {:.0}%", self.gpu_usage)).into(),
                        ]).spacing(8)
                    ),
            )
            .add(
                cosmic::widget::settings::item::builder("Memory")
                    .control(
                        widget::row::with_children(vec![
                            widget::progress_bar(0.0..=100.0, memory_percent).into(),
                            widget::text::body(format!(" {:.0}%", memory_percent)).into(),
                        ]).spacing(8)
                    ),
            )
            .add(
                cosmic::widget::settings::item::builder("Disk")
                    .control(
                        widget::row::with_children(vec![
                            widget::progress_bar(0.0..=100.0, disk_percent).into(),
                            widget::text::body(format!(" {:.0}%", disk_percent)).into(),
                        ]).spacing(8)
                    ),
            );

        let processes_section = cosmic::widget::settings::section()
            .title("Processes")
            .add(
                cosmic::widget::settings::item::builder("Total")
                    .control(widget::text::body(format!("{}", self.sys.processes().len()))),
            );

        widget::column::with_capacity(4)
            .push(widget::text::title1("System Information"))
            .push(
                cosmic::widget::settings::section()
                    .title(format!("{}@{}", self.username, self.hostname))
                    .add(info_row)
            )
            .push(usage_section)
            .push(processes_section)
            .spacing(space_s)
            .into()
    }

    fn cpu_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 16;
        let cores = self.sys.cpus().len();
        let freq = self.sys.cpus().first().map(|c: &sysinfo::Cpu| c.frequency()).unwrap_or(0);

        let header = widget::text::title1("CPU Information");

        let info_section = cosmic::widget::settings::section()
            .title("Details")
            .add(
                cosmic::widget::settings::item::builder("Cores")
                    .control(widget::text::body(cores.to_string())),
            )
            .add(
                cosmic::widget::settings::item::builder("Frequency")
                    .control(widget::text::body(format!("{} MHz", freq))),
            );

        let usage_bar = widget::progress_bar(0.0..=100.0, self.cpu_usage);

        let usage_section = cosmic::widget::settings::section()
            .title("Usage")
            .add(
                cosmic::widget::settings::item::builder("Overall")
                    .control(widget::text::heading(format!("{:.1}%", self.cpu_usage))),
            )
            .add(
                cosmic::widget::settings::item::builder("Visual")
                    .control(usage_bar),
            );

        let mut core_items: Vec<Element<'_, Message>> = Vec::new();
        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            let cpu_usage = cpu.cpu_usage();
            let bar = widget::progress_bar(0.0..=100.0, cpu_usage);
            core_items.push(
                widget::row::with_capacity(3)
                    .width(Length::Fill)
                    .push(widget::text::body(format!("Core {}", i)))
                    .push(bar)
                    .push(widget::text::body(format!("{:.0}%", cpu_usage)))
                    .spacing(12)
                    .into(),
            );
        }

        let mut core_column = widget::column::with_capacity(core_items.len());
        for item in core_items {
            core_column = core_column.push(item);
        }
        core_column = core_column.spacing(8);

        let core_section = cosmic::widget::settings::section()
            .title("Per-Core Usage")
            .add(core_column);

        widget::column::with_capacity(4)
            .push(header)
            .push(info_section)
            .push(usage_section)
            .push(core_section)
            .spacing(space_s)
            .into()
    }

    fn memory_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 24;
        let used_gb = self.memory_used as f64 / 1_073_741_824.0;
        let total_gb = self.memory_total as f64 / 1_073_741_824.0;
        let available_gb = (self.memory_total - self.memory_used) as f64 / 1_073_741_824.0;
        let percent = if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let header = widget::text::title1("Memory Information");

        let usage_bar = widget::progress_bar(0.0..=100.0, percent as f32);

        let hero_section = widget::container(
            widget::column::with_capacity(3)
                .spacing(12)
                .push(widget::text::title1(format!("{:.1}%", percent)))
                .push(widget::text::heading(format!(
                    "{:.1} GB used of {:.1} GB",
                    used_gb, total_gb
                )))
                .push(usage_bar)
        )
        .width(Length::Fill)
        .padding(32);

        let info_section = cosmic::widget::settings::section()
            .title("Details")
            .add(
                cosmic::widget::settings::item::builder("Used")
                    .control(widget::text::body(format!("{:.2} GB", used_gb))),
            )
            .add(
                cosmic::widget::settings::item::builder("Available")
                    .control(widget::text::body(format!("{:.2} GB", available_gb))),
            )
            .add(
                cosmic::widget::settings::item::builder("Total")
                    .control(widget::text::body(format!("{:.2} GB", total_gb))),
            );

        widget::column::with_capacity(3)
            .push(header)
            .push(hero_section)
            .push(info_section)
            .spacing(space_s)
            .into()
    }

    fn processes_view(&self) -> Element<'_, Message> {
        let mut processes: Vec<_> = self.sys.processes().iter().collect();

        match self.process_sort {
            ProcessSort::Cpu => {
                processes.sort_by(|a, b| {
                    let cpu_a = a.1.cpu_usage();
                    let cpu_b = b.1.cpu_usage();
                    cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProcessSort::Memory => {
                processes.sort_by(|a, b| {
                    let mem_a = a.1.memory();
                    let mem_b = b.1.memory();
                    mem_b.cmp(&mem_a)
                });
            }
            ProcessSort::Alphabetical => {
                processes.sort_by(|a, b| {
                    let name_a = a.1.name().to_string_lossy().to_lowercase();
                    let name_b = b.1.name().to_string_lossy().to_lowercase();
                    name_a.cmp(&name_b)
                });
            }
        }
        processes.truncate(20);

        let max_memory = processes.iter()
            .map(|(_, p)| p.memory() as f64)
            .fold(1.0f64, |a, b| a.max(b));

        let count = processes.len();
        
        let sort_label = match self.process_sort {
            ProcessSort::Cpu => "CPU %",
            ProcessSort::Memory => "Memory",
            ProcessSort::Alphabetical => "Name",
        };

        let name_col_header = widget::button::standard("Name")
            .on_press(Message::SortProcesses(ProcessSort::Alphabetical))
            .width(Length::Fixed(250.0));

        let cpu_col_header = widget::button::standard("CPU %")
            .on_press(Message::SortProcesses(ProcessSort::Cpu))
            .width(Length::Fill);

        let mem_col_header = widget::button::standard("Memory")
            .on_press(Message::SortProcesses(ProcessSort::Memory))
            .width(Length::Fill);

        let mut name_items: Vec<Element<_>> = vec![name_col_header.into()];
        let mut cpu_items: Vec<Element<_>> = vec![cpu_col_header.into()];
        let mut mem_items: Vec<Element<_>> = vec![mem_col_header.into()];

        for (_pid, process) in &processes {
            let name: String = process.name().to_string_lossy().chars().take(25).collect();
            let cpu = process.cpu_usage();
            let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
            let memory_percent = (process.memory() as f64 / max_memory * 100.0) as f32;
            let memory_bar = widget::progress_bar(0.0..=100.0, memory_percent);
            
            let memory_str = if memory_mb >= 1024.0 {
                format!("{:.1} GB", memory_mb / 1024.0)
            } else {
                format!("{:.0} MB", memory_mb)
            };

            name_items.push(widget::text::body(name).into());
            cpu_items.push(widget::text::body(format!("{:.1}%", cpu)).into());
            mem_items.push(
                widget::row::with_children(vec![
                    memory_bar.into(),
                    widget::text::body(memory_str).into(),
                ]).spacing(8).into()
            );
        }

        let name_col = widget::column::with_children(name_items).spacing(8).width(Length::Fixed(250.0));

        let cpu_col = widget::column::with_children(cpu_items).spacing(8).width(Length::Fixed(80.0));

        let mem_col = widget::column::with_children(mem_items).spacing(8).width(Length::Fill);

        let scrollable_content: Element<_> = widget::row::with_capacity(3)
            .spacing(16)
            .push(name_col)
            .push(cpu_col)
            .push(mem_col)
            .into();

        let scrollable = widget::scrollable(scrollable_content)
            .height(Length::Fill);

        widget::column::with_capacity(3)
            .push(widget::text::title1(format!("Top Processes")))
            .push(widget::text::body(format!("Showing {} of {} processes (sorted by {})", count, self.sys.processes().len(), sort_label)))
            .push(scrollable)
            .spacing(12)
            .into()
    }

    fn network_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 24;
        let header = widget::text::title1("Network Statistics");

        let mut total_received: u64 = 0;
        let mut total_transmitted: u64 = 0;

        for (_name, data) in self.networks.iter() {
            total_received += data.total_received();
            total_transmitted += data.total_transmitted();
        }

        let format_bytes = |bytes: u64| -> String {
            let kb = bytes as f64 / 1024.0;
            let mb = kb / 1024.0;
            let gb = mb / 1024.0;
            if gb >= 1.0 {
                format!("{:.2} GB", gb)
            } else if mb >= 1.0 {
                format!("{:.2} MB", mb)
            } else {
                format!("{:.2} KB", kb)
            }
        };

        let totals_hero = widget::container(
            widget::row::with_capacity(2)
                .spacing(48)
                .push(
                    widget::column::with_capacity(2)
                        .spacing(8)
                        .push(
                            widget::row::with_capacity(2)
                                .spacing(8)
                                .push(icon::from_name("go-down-symbolic").size(24))
                                .push(widget::text::title1(format!("{}", format_bytes(total_received))))
                        )
                        .push(widget::text::body("Downloaded"))
                )
                .push(
                    widget::column::with_capacity(2)
                        .spacing(8)
                        .push(
                            widget::row::with_capacity(2)
                                .spacing(8)
                                .push(icon::from_name("go-up-symbolic").size(24))
                                .push(widget::text::title1(format!("{}", format_bytes(total_transmitted))))
                        )
                        .push(widget::text::body("Uploaded"))
                )
        )
        .width(Length::Fill)
        .padding(24);

        widget::column::with_capacity(2)
            .push(header)
            .push(totals_hero)
            .spacing(space_s)
            .into()
    }

    fn disks_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 24;
        let header = widget::text::title1("Disk Usage");

        let format_bytes = |bytes: u64| -> String {
            let gb = bytes as f64 / 1_073_741_824.0;
            let tb = gb / 1024.0;
            if tb >= 1.0 {
                format!("{:.1} TB", tb)
            } else {
                format!("{:.1} GB", gb)
            }
        };

        let mut disk_items: Vec<Element<'_, Message>> = Vec::new();

        for disk in self.disks.iter() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let percent = if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            };

            let usage_bar = widget::progress_bar(0.0..=100.0, percent);

            let disk_card = widget::container(
                widget::column::with_capacity(4)
                    .spacing(12)
                    .push(
                        widget::row::with_capacity(3)
                            .spacing(8)
                            .push(icon::from_name("drive-harddisk-symbolic").size(24))
                            .push(widget::text::heading(mount_point.clone()))
                            .push(widget::text::body(format!("({})", disk.name().to_string_lossy())))
                    )
                    .push(
                        widget::row::with_capacity(3)
                            .spacing(16)
                            .push(widget::text::body(format!("Used: {}", format_bytes(used))))
                            .push(widget::text::body(format!("Avail: {}", format_bytes(available))))
                            .push(widget::text::body(format!("Total: {}", format_bytes(total))))
                    )
                    .push(
                        widget::row::with_capacity(2)
                            .spacing(8)
                            .push(usage_bar)
                            .push(widget::text::body(format!("{:.0}%", percent)))
                    )
            )
            .width(Length::Fill)
            .padding(16);

            disk_items.push(disk_card.into());
        }

        let mut disk_column = widget::column::with_capacity(disk_items.len());
        for item in disk_items {
            disk_column = disk_column.push(item);
        }

        widget::column::with_capacity(2)
            .push(header)
            .push(disk_column.spacing(space_s))
            .spacing(space_s)
            .into()
    }

    fn refresh_gpu_info(&mut self) {
        let output = std::process::Command::new("nvidia-smi")
            .arg("--query-gpu=name,utilization.gpu,memory.used,memory.total")
            .arg("--format=csv,noheader,nounits")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        self.gpu_name = parts[0].to_string();
                        self.gpu_usage = parts[1].parse().unwrap_or(0.0);
                        self.gpu_memory_used = parts[2].parse().unwrap_or(0);
                        self.gpu_memory_total = parts[3].parse().unwrap_or(0);
                    }
                }
            }
        }

        if self.gpu_name.is_empty() {
            self.gpu_name = "No GPU detected".to_string();
        }
    }

    fn gpu_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 16;

        let header = widget::text::title1("GPU Information");

        let info_section = cosmic::widget::settings::section()
            .title("Details")
            .add(
                cosmic::widget::settings::item::builder("Name")
                    .control(widget::text::body(self.gpu_name.clone())),
            );

        let gpu_memory_percent = if self.gpu_memory_total > 0 {
            (self.gpu_memory_used as f64 / self.gpu_memory_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let usage_bar = widget::progress_bar(0.0..=100.0, self.gpu_usage);
        let memory_bar = widget::progress_bar(0.0..=100.0, gpu_memory_percent);

        let usage_section = cosmic::widget::settings::section()
            .title("Usage")
            .add(
                cosmic::widget::settings::item::builder("GPU")
                    .control(
                        widget::row::with_children(vec![
                            usage_bar.into(),
                            widget::text::body(format!(" {:.1}%", self.gpu_usage)).into(),
                        ]).spacing(8)
                    ),
            )
            .add(
                cosmic::widget::settings::item::builder("Memory")
                    .control(
                        widget::row::with_children(vec![
                            memory_bar.into(),
                            widget::text::body(format!(" {:.0}%", gpu_memory_percent)).into(),
                        ]).spacing(8)
                    ),
            )
            .add(
                cosmic::widget::settings::item::builder("Memory Used")
                    .control(widget::text::body(format!("{} / {} MB", self.gpu_memory_used, self.gpu_memory_total))),
            );

        widget::column::with_capacity(3)
            .push(header)
            .push(info_section)
            .push(usage_section)
            .spacing(space_s)
            .into()
    }
}

pub enum Page {
    Overview,
    Cpu,
    Memory,
    Processes,
    Network,
    Disks,
    Gpu,
}
