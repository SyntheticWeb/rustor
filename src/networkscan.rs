use log::info;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, List, ListState, Padding, Paragraph},
    Frame,
};

use sysinfo::Networks;

use pistol;

use crate::app::{App, AppInfo, AppMessage};
use core::{fmt, net};
use std::{
    collections::HashMap,
    fmt::{write, Debug},
    net::Ipv4Addr,
};

use crossterm::event;
use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct NetScanApp {
    info: AppInfo,
    text: std::string::String,
    interfaces: NetInterfaces,
    selected_interface: ListState,
    current_options: ScanOptions,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    icmp_host_ping: bool,
}

#[derive(Debug, Clone)]
struct NetInterfaces {
    networks: Vec<NetInfo>,
}

#[derive(Debug, Clone)]
struct NetInfo {
    ip4: Option<net::Ipv4Addr>,
    ip6: Option<net::Ipv6Addr>,
    nmask: Option<net::IpAddr>,
    mac: String,
    name: String,
}

enum OptionEnable {
    ICMPHost
}

pub enum NetScanMsg {
    Scan,
    NextInterface,
}

impl AppMessage for NetScanMsg {}

impl App for NetScanApp {
    type Msg = NetScanMsg;

    fn view(&mut self, layout: &Layout, frame: &mut Frame, style: Style) {
        let app_area = layout.split(frame.area())[1];

        let vertical = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let interface_split =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]);

        let scan_split =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]);

        let [interface_area, scan_area] = vertical.areas(app_area);

        let [list_interface_area, info_area] = interface_split.areas(interface_area);
        let [scan_options_area, scan_results_area] = scan_split.areas(scan_area);

        let interface_entries: Vec<String> = self
            .interface_names()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index != self.selected_interface.selected().unwrap() {
                    format!("[ ] {:<3}", item)
                } else {
                    format!("[*] {:<3}", item)
                }
            })
            .collect();

        let interface_list = List::new(interface_entries)
            .style(style)
            .block(Block::bordered().title("Interfaces (i - next interface)"));

        frame.render_stateful_widget(
            interface_list,
            list_interface_area,
            &mut self.selected_interface,
        );

        let interface_info_box = Paragraph::new(self.selected_interface_info())
            .style(style)
            .block(Block::bordered().title("Info"));

        frame.render_widget(interface_info_box, info_area);

        let scan_options_box = Paragraph::new("Scan options go here")
            .style(style)
            .block(Block::bordered().title("Scan Options"));

        frame.render_widget(scan_options_box, scan_options_area);

        let scan_results_box = Paragraph::new("Scan results go here")
            .style(style)
            .block(Block::bordered().title("Scan Results"));

        frame.render_widget(scan_results_box, scan_results_area);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            NetScanMsg::Scan => self.text = "Event Received!".to_string(),
            NetScanMsg::NextInterface => self.next_interface(),
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }

    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match key_event.code {
            KeyCode::Enter => Some(NetScanMsg::Scan),
            KeyCode::Char('i') => Some(NetScanMsg::NextInterface),
            _ => None,
        }
    }
}

impl NetScanApp {
    pub fn new() -> NetScanApp {
        let net_interfaces = NetInterfaces::new();

        let main_app = NetScanApp {
            info: AppInfo {
                title: "Netscan".to_string(),
                version: "v1.0".to_string(),
            },
            text: "Network app for scanning network interfaces".to_string(),
            interfaces: net_interfaces,
            selected_interface: ListState::default().with_selected(Some(0)),
            current_options: ScanOptions{
                icmp_host_ping: false,
            }

        };

        return main_app;
    }

    pub fn read_interfaces(&mut self) {
        let net_interfaces = NetInterfaces::new();

        self.interfaces = net_interfaces;
    }

    pub fn next_interface(&mut self) {
        if self.selected_interface.selected().unwrap() != self.interface_names().len() - 1 {
            self.selected_interface.select_next();
        } else {
            self.selected_interface.select_first();
        }
    }

    pub fn set_option(&mut self, option ) {}

    pub fn interface_names(&self) -> Vec<String> {
        let mut interface_names: Vec<String> = vec![];

        for network in &self.interfaces.networks {
            interface_names.push(network.name.clone());
        }

        return interface_names;
    }

    pub fn selected_interface_info(&self) -> String {
        return format!(
            "{}",
            self.interfaces.networks[self.selected_interface.selected().unwrap()]
        );
    }

    pub fn current_interface(&self) -> String {
        return self.interfaces.networks[self.selected_interface.selected().unwrap()]
            .name
            .clone();
    }

    pub fn scan_hosts(mut& self) {
        if self
    }

    pub fn icmp_scan() {

    }
}

impl NetInterfaces {
    fn new() -> NetInterfaces {
        let networks = Networks::new_with_refreshed_list();
        let mut net_interfaces = Vec::new();

        for (name, data) in &networks {
            let ip_networks = data.ip_networks();
            let mac = data.mac_address();

            let mut info = NetInfo {
                mac: mac.to_string(),
                ip4: None,
                ip6: None,
                nmask: None,
                name: name.clone(),
            };

            for ip in ip_networks {
                match ip.addr {
                    std::net::IpAddr::V4(addr) => info.ip4 = Some(addr),
                    std::net::IpAddr::V6(addr) => info.ip6 = Some(addr),
                }
            }

            net_interfaces.push(info);
        }

        return NetInterfaces {
            networks: net_interfaces,
        };
    }
}

impl fmt::Display for NetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IP4: {:?}\nIP6: {:?}\nNetmask: {:?}\nMAC: {}",
            self.ip4, self.ip6, self.nmask, self.mac
        )
    }
}
