// SPDX-License-Identifier: GPL-v3

use cosmic::app::context_drawer;
use cosmic::iced::{Length, Subscription};
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{iced_futures, prelude::*};
use cosmic::iced::futures::SinkExt;
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{Disks, Networks, System};

const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    core: cosmic::Core,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    sys: System,
    networks: Networks,
    disks: Disks,
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    RefreshSystemInfo,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.zachvlat.system-monitor";

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

        let mut app = AppModel {
            core,
            nav,
            key_binds: HashMap::new(),
            sys,
            networks,
            disks,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total,
        };

        let command = app.update_title();
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root("File").apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button("About", None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
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
            }
            Message::LaunchUrl(url) => {
                let _ = open::that_detached(&url);
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
        let space_s: u16 = 12;

        let cpu_section = cosmic::widget::settings::section()
            .title("CPU")
            .add(
                cosmic::widget::settings::item::builder("Usage")
                    .control(widget::text::body(format!("{:.1}%", self.cpu_usage))),
            );

        let memory_used_gb = self.memory_used as f64 / 1_073_741_824.0;
        let memory_total_gb = self.memory_total as f64 / 1_073_741_824.0;
        let memory_percent = if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let memory_section = cosmic::widget::settings::section()
            .title("Memory")
            .add(
                cosmic::widget::settings::item::builder("Used")
                    .control(widget::text::body(format!(
                        "{:.1} GB / {:.1} GB ({:.1}%)",
                        memory_used_gb, memory_total_gb, memory_percent
                    ))),
            );

        let processes_section = cosmic::widget::settings::section()
            .title("Processes")
            .add(
                cosmic::widget::settings::item::builder("Total")
                    .control(widget::text::body(format!("{}", self.sys.processes().len()))),
            );

        widget::column::with_capacity(4)
            .push(widget::text::title1("System Overview"))
            .push(cpu_section)
            .push(memory_section)
            .push(processes_section)
            .spacing(space_s)
            .into()
    }

    fn cpu_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 12;
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

        let usage_section = cosmic::widget::settings::section()
            .title("Usage")
            .add(
                cosmic::widget::settings::item::builder("Overall")
                    .control(widget::text::body(format!("{:.1}%", self.cpu_usage))),
            );

        let mut core_items: Vec<Element<'_, Message>> = Vec::new();
        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            let cpu_usage = cpu.cpu_usage();
            core_items.push(
                cosmic::widget::settings::item::builder(format!("Core {}", i))
                    .control(widget::text::body(format!("{:.1}%", cpu_usage)))
                    .into(),
            );
        }

        let mut core_column = widget::column::with_capacity(core_items.len());
        for item in core_items {
            core_column = core_column.push(item);
        }
        core_column = core_column.spacing(space_s);

        let core_section = cosmic::widget::settings::section()
            .title("Per-Core Usage");

        widget::column::with_capacity(5)
            .push(header)
            .push(info_section)
            .push(usage_section)
            .push(core_section)
            .push(core_column)
            .spacing(space_s)
            .into()
    }

    fn memory_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 12;
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

        let info_section = cosmic::widget::settings::section()
            .title("Usage")
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
            )
            .add(
                cosmic::widget::settings::item::builder("Percentage")
                    .control(widget::text::body(format!("{:.1}%", percent))),
            )
            .add(
                cosmic::widget::settings::item::builder("Visual")
                    .control(usage_bar),
            );

        widget::column::with_capacity(2)
            .push(header)
            .push(info_section)
            .spacing(space_s)
            .into()
    }

    fn processes_view(&self) -> Element<'_, Message> {
        let mut processes: Vec<_> = self.sys.processes().iter().collect();
        processes.sort_by(|a, b| {
            let cpu_a = a.1.cpu_usage();
            let cpu_b = b.1.cpu_usage();
            cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(20);

        let count = processes.len();
        let mut column = widget::column::with_capacity(processes.len());
        
        for (_pid, process) in &processes {
            let name: String = process.name().to_string_lossy().chars().take(30).collect();
            let cpu = process.cpu_usage();
            let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
            let memory_str = if memory_mb >= 1024.0 {
                format!("{:.1} GB", memory_mb / 1024.0)
            } else {
                format!("{:.0} MB", memory_mb)
            };
            let process_text = format!("{:<30} {:>6.1}%    {}", name, cpu, memory_str);
            column = column.push(widget::text::body(process_text));
        }

        widget::column::with_capacity(2)
            .push(widget::text::title1("Top Processes (by CPU)"))
            .push(cosmic::widget::settings::section()
                .title(format!("{} processes shown", count))
                .add(column.spacing(4)))
            .spacing(12)
            .into()
    }

    fn network_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 12;
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

        let totals_section = cosmic::widget::settings::section()
            .title("Total Since Boot")
            .add(
                cosmic::widget::settings::item::builder("Received")
                    .control(widget::text::body(format!("{}", format_bytes(total_received)))),
            )
            .add(
                cosmic::widget::settings::item::builder("Transmitted")
                    .control(widget::text::body(format!("{}", format_bytes(total_transmitted)))),
            );

        let mut interface_items: Vec<Element<'_, Message>> = Vec::new();
        let mut interface_count = 0;

        for (name, data) in self.networks.iter() {
            if interface_count >= 10 {
                break;
            }
            interface_items.push(
                cosmic::widget::settings::item::builder(name)
                    .control(
                        widget::text::body(format!(
                            "↓ {}  ↑ {}",
                            format_bytes(data.total_received()),
                            format_bytes(data.total_transmitted())
                        )),
                    )
                    .into(),
            );
            interface_count += 1;
        }

        let mut interface_column = widget::column::with_capacity(interface_items.len());
        for item in interface_items {
            interface_column = interface_column.push(item);
        }

        let interface_section = cosmic::widget::settings::section()
            .title("Interfaces")
            .add(interface_column);

        widget::column::with_capacity(3)
            .push(header)
            .push(totals_section)
            .push(interface_section)
            .spacing(space_s)
            .into()
    }

    fn disks_view(&self) -> Element<'_, Message> {
        let space_s: u16 = 12;
        let header = widget::text::title1("Disk Usage");

        let format_bytes = |bytes: u64| -> String {
            let gb = bytes as f64 / 1_073_741_824.0;
            let tb = gb / 1024.0;
            if tb >= 1.0 {
                format!("{:.2} TB", tb)
            } else {
                format!("{:.2} GB", gb)
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

            let disk_info = cosmic::widget::settings::section()
                .title(format!("{} ({})", mount_point, disk.name().to_string_lossy()))
                .add(
                    cosmic::widget::settings::item::builder("Used")
                        .control(widget::text::body(format!("{} / {}", format_bytes(used), format_bytes(total)))),
                )
                .add(
                    cosmic::widget::settings::item::builder("Available")
                        .control(widget::text::body(format_bytes(available))),
                )
                .add(
                    cosmic::widget::settings::item::builder("Usage")
                        .control(usage_bar),
                );

            disk_items.push(disk_info.into());
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
}

pub enum Page {
    Overview,
    Cpu,
    Memory,
    Processes,
    Network,
    Disks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::LaunchUrl(String::new())
    }
}
