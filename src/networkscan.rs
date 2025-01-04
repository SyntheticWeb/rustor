use log::info;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListState, Padding, Paragraph},
    Frame,
};

use pnet::{datalink, ipnetwork};
use sysinfo::Networks;

use pistol::{self, Target};

use crate::{
    app::{App, AppInfo, AppMessage},
    components::optionlist::OptionListState,
};
use core::{fmt, net};
use std::{
    collections::HashMap,
    fmt::{write, Debug},
    iter::Scan,
    net::{IpAddr, Ipv4Addr},
};

use crossterm::event;
use crossterm::event::KeyCode;

use crate::components::OptionList;

#[derive(Debug, Clone)]
pub struct NetScanApp {
    info: AppInfo,
    text: std::string::String,
    interfaces: NetInterfaces,
    selected_interface: ListState,
    current_options: ScanOptions,
    options_state: OptionListState,
    scan_results: String,
}

#[derive(Debug, Clone)]
struct ScanOptions {
    icmp_ping: ScanOption,
    port_scan: ScanOption,
    names: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScanOption {
    name: String,
    index: usize,
    active: bool,
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
    ICMPHost,
}

pub enum NetScanMsg {
    Scan,
    NextInterface,
    NextScanOption,
    ToggleScanOption,
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

        let scan_options = OptionList::new(
            self.current_options.names.clone(),
            "[*]".to_string(),
            "[ ]".to_string(),
            "Scan Options".to_string(),
            Style::default().bg(Color::Green).fg(Color::White),
            style,
        );

        frame.render_stateful_widget(scan_options, scan_options_area, &mut self.options_state);

        let scan_results_box = Paragraph::new("Scan results go here")
            .style(style)
            .block(Block::bordered().title("Scan Results"));

        frame.render_widget(scan_results_box, scan_results_area);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            NetScanMsg::Scan => self.scan_hosts(),
            NetScanMsg::NextInterface => self.next_interface(),
            NetScanMsg::NextScanOption => self.next_scan_option(),
            NetScanMsg::ToggleScanOption => self.set_selected_option(),
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }

    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match key_event.code {
            KeyCode::Enter => Some(NetScanMsg::Scan),
            KeyCode::Char('i') => Some(NetScanMsg::NextInterface),
            KeyCode::Char('s') => Some(NetScanMsg::Scan),
            KeyCode::Char('o') => Some(NetScanMsg::NextScanOption),
            KeyCode::Char('O') => Some(NetScanMsg::ToggleScanOption),
            _ => None,
        }
    }
}

impl NetScanApp {
    pub fn new() -> NetScanApp {
        let net_interfaces = NetInterfaces::new();

        let icmp_option = ScanOption {
            name: "ICMP Scan".to_string(),
            index: 0,
            active: false,
        };

        let port_option = ScanOption {
            name: "Port Scan".to_string(),
            index: 1,
            active: false,
        };

        let option_names = vec!["ICMP Ping".to_string(), "Port Scan".to_string()];

        let main_app = NetScanApp {
            info: AppInfo {
                title: "Netscan".to_string(),
                version: "v1.0".to_string(),
            },
            text: "Network app for scanning network interfaces".to_string(),
            interfaces: net_interfaces,
            selected_interface: ListState::default().with_selected(Some(0)),
            options_state: OptionListState::new(option_names.len()),
            current_options: ScanOptions {
                icmp_ping: icmp_option,
                port_scan: port_option,
                names: option_names,
            },
            scan_results: "".to_string(),
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

    pub fn set_selected_option(&mut self) {
        match self.options_state.highlighted {
            Some(0) => {
                self.current_options.icmp_ping.active = !self.current_options.icmp_ping.active;
                if self.current_options.icmp_ping.active {
                    self.options_state.select()
                } else {
                    self.options_state.unselect()
                }
            }
            _ => {}
        }
    }

    pub fn next_scan_option(&mut self) {
        self.options_state.highlight_next()
    }

    pub fn prev_scan_option(&mut self) {
        self.options_state.highlight_prev()
    }

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

    pub fn current_target(&self) -> Target {
        let netinfo = self.interfaces.networks[self.selected_interface.selected().unwrap()].clone();

        let cidr_number = calculate_cidr(netinfo.nmask.unwrap());

        return Target::from_subnet("192.168.1.1/24", None).unwrap();
    }

    pub fn scan_hosts(&mut self) {
        let mut result = String::new();
        if self.current_options.icmp_ping.active {
            self.icmp_scan();
        }
    }

    pub fn icmp_scan(&self) -> String {
        let target = self.current_target();

        return "".to_string();
    }
}

fn calculate_cidr(nmask: IpAddr) -> usize {
    let octets = nmask.to_string();

    let mut cidr = 0;

    for octet in octets.split('.').into_iter() {
        let num = octet.parse::<usize>().unwrap();

        cidr += num.count_ones();
    }

    info!("Calculated CIDR: {}", cidr);

    return cidr.try_into().unwrap();
}

impl NetInterfaces {
    fn new() -> NetInterfaces {
        let networks = datalink::interfaces();
        let mut net_interfaces = Vec::new();

        for interface in &networks {
            let ip_networks = interface.ips.clone();
            let mut mac_string = "".to_string();
            if let Some(mac) = interface.mac {
                mac_string = mac.to_string();
            }

            let mut info = NetInfo {
                mac: mac_string.clone(),
                ip4: None,
                ip6: None,
                nmask: None,
                name: interface.name.clone(),
            };

            for ip in ip_networks {
                match ip.ip() {
                    std::net::IpAddr::V4(addr) => {
                        info.ip4 = Some(addr);
                        info.nmask = Some(ip.mask())
                    }
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
        let mut ip4_string = String::new();
        let mut ip6_string = String::new();
        let mut nmask_string = String::new();

        match self.ip4 {
            Some(ip) => ip4_string = ip.to_string(),
            None => ip4_string = "N/A".to_string(),
        }

        match self.ip6 {
            Some(ip) => ip6_string = ip.to_string(),
            None => ip6_string = "N/A".to_string(),
        }

        match self.nmask {
            Some(mask) => nmask_string = mask.to_string(),
            None => nmask_string = "N/A".to_string(),
        }

        write!(
            f,
            "IP4: {}\nIP6: {}\nNetmask: {}\nMAC: {}",
            ip4_string, ip6_string, nmask_string, self.mac
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cidr() {
        let nmask = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0));

        let result = calculate_cidr(nmask);
        assert_eq!(result, 24);
    }
}
