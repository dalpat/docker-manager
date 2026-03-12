use crate::docker::DockerClient;
use crate::model::ContainerInfo;
use adw::prelude::*;
use glib::{Continue, MainContext, PRIORITY_DEFAULT};
use gtk::{
    Align, Box as GtkBox, Button, CheckButton, Entry, Image, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, SelectionMode, TextBuffer, TextView, WrapMode,
};
use std::thread;

enum Action {
    Refresh,
    Start(String),
    Stop(String),
    Remove(String),
}

struct WorkerMessage {
    headline: String,
    details: Result<String, String>,
    containers: Option<Vec<ContainerInfo>>,
}

pub fn run() {
    let app = adw::Application::builder()
        .application_id("io.github.dalpat.dockermanager")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Docker Manager")
        .default_width(1080)
        .default_height(760)
        .build();
    window.set_icon_name(Some("io.github.dalpat.dockermanager"));
    window.set_decorated(true);
    window.set_resizable(true);
    window.set_deletable(true);

    let header = gtk::HeaderBar::new();
    header.set_show_title_buttons(true);
    window.set_titlebar(Some(&header));

    let root = GtkBox::new(Orientation::Vertical, 14);
    root.set_margin_top(18);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);

    let header = GtkBox::new(Orientation::Vertical, 4);
    let title = Label::new(Some("Docker Command Center"));
    title.set_halign(Align::Start);
    title.add_css_class("title-1");

    let subtitle = Label::new(Some(
        "Production-ready desktop workflow for inspecting and controlling containers.",
    ));
    subtitle.set_halign(Align::Start);
    subtitle.add_css_class("dim-label");

    header.append(&title);
    header.append(&subtitle);
    root.append(&header);

    let controls = GtkBox::new(Orientation::Vertical, 8);
    let target_entry = Entry::builder()
        .placeholder_text("Container name or ID")
        .hexpand(true)
        .build();
    controls.append(&target_entry);

    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    let refresh_btn = action_button("Refresh", "view-refresh-symbolic");
    let start_btn = action_button("Start", "media-playback-start-symbolic");
    let stop_btn = action_button("Stop", "media-playback-stop-symbolic");
    let remove_btn = action_button("Remove", "user-trash-symbolic");
    let exit_btn = action_button("Exit", "window-close-symbolic");
    refresh_btn.add_css_class("suggested-action");
    remove_btn.add_css_class("destructive-action");

    buttons.append(&refresh_btn);
    buttons.append(&start_btn);
    buttons.append(&stop_btn);
    buttons.append(&remove_btn);
    buttons.append(&exit_btn);
    controls.append(&buttons);

    let danger_check = CheckButton::with_label("Enable destructive actions");
    danger_check.set_halign(Align::Start);
    controls.append(&danger_check);
    root.append(&controls);

    let body = GtkBox::new(Orientation::Horizontal, 12);
    body.set_vexpand(true);

    let container_panel = GtkBox::new(Orientation::Vertical, 8);
    container_panel.set_hexpand(true);
    container_panel.set_vexpand(true);

    let container_title = Label::new(Some("Containers"));
    container_title.set_halign(Align::Start);
    container_title.add_css_class("heading");
    container_panel.append(&container_title);

    let container_list = ListBox::new();
    container_list.set_selection_mode(SelectionMode::Single);
    container_list.add_css_class("boxed-list");
    container_list.set_vexpand(true);
    let container_scroller = ScrolledWindow::builder()
        .child(&container_list)
        .hexpand(true)
        .vexpand(true)
        .build();
    container_panel.append(&container_scroller);

    body.append(&container_panel);

    let log_panel = GtkBox::new(Orientation::Vertical, 8);
    log_panel.set_hexpand(true);
    log_panel.set_vexpand(true);

    let log_title = Label::new(Some("Activity Log"));
    log_title.set_halign(Align::Start);
    log_title.add_css_class("heading");
    log_panel.append(&log_title);

    let log_view = TextView::new();
    log_view.set_editable(false);
    log_view.set_monospace(true);
    log_view.set_wrap_mode(WrapMode::WordChar);
    log_view.set_vexpand(true);
    let log_buffer = TextBuffer::new(None);
    log_buffer.set_text("Application started.\n");
    log_view.set_buffer(Some(&log_buffer));

    let log_scroller = ScrolledWindow::builder()
        .child(&log_view)
        .hexpand(true)
        .vexpand(true)
        .build();
    log_panel.append(&log_scroller);

    body.append(&log_panel);
    root.append(&body);

    let status = Label::new(Some("Status: Ready"));
    status.set_halign(Align::Start);
    status.add_css_class("dim-label");
    root.append(&status);

    window.set_child(Some(&root));

    let action_buttons = vec![
        refresh_btn.clone(),
        start_btn.clone(),
        stop_btn.clone(),
        remove_btn.clone(),
    ];

    let (sender, receiver) = MainContext::channel::<WorkerMessage>(PRIORITY_DEFAULT);
    let receiver_status = status.clone();
    let receiver_buffer = log_buffer.clone();
    let receiver_list = container_list.clone();
    let receiver_buttons = action_buttons.clone();
    receiver.attach(None, move |message| {
        set_buttons_enabled(&receiver_buttons, true);
        receiver_status.set_text("Status: Ready");

        append_log(&receiver_buffer, &message.headline, message.details);
        if let Some(containers) = message.containers {
            populate_container_list(&receiver_list, &containers);
        }
        Continue(true)
    });

    let list_entry = target_entry.clone();
    container_list.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            if let Some(name) = row.tooltip_text() {
                list_entry.set_text(name.as_str());
            }
        }
    });

    let window_for_exit = window.clone();
    exit_btn.connect_clicked(move |_| {
        window_for_exit.close();
    });

    let refresh_sender = sender.clone();
    let refresh_status = status.clone();
    let refresh_buttons = action_buttons.clone();
    let refresh_buffer = log_buffer.clone();
    refresh_btn.connect_clicked(move |_| {
        run_action(
            Action::Refresh,
            refresh_sender.clone(),
            &refresh_status,
            &refresh_buttons,
            &refresh_buffer,
        );
    });

    let start_sender = sender.clone();
    let start_status = status.clone();
    let start_buttons = action_buttons.clone();
    let start_buffer = log_buffer.clone();
    let start_entry = target_entry.clone();
    start_btn.connect_clicked(move |_| {
        let target = start_entry.text().trim().to_string();
        if target.is_empty() {
            append_log(
                &start_buffer,
                "Start container",
                Err("Container name or ID is required.".to_string()),
            );
            return;
        }

        run_action(
            Action::Start(target),
            start_sender.clone(),
            &start_status,
            &start_buttons,
            &start_buffer,
        );
    });

    let stop_sender = sender.clone();
    let stop_status = status.clone();
    let stop_buttons = action_buttons.clone();
    let stop_buffer = log_buffer.clone();
    let stop_entry = target_entry.clone();
    stop_btn.connect_clicked(move |_| {
        let target = stop_entry.text().trim().to_string();
        if target.is_empty() {
            append_log(
                &stop_buffer,
                "Stop container",
                Err("Container name or ID is required.".to_string()),
            );
            return;
        }

        run_action(
            Action::Stop(target),
            stop_sender.clone(),
            &stop_status,
            &stop_buttons,
            &stop_buffer,
        );
    });

    let remove_sender = sender.clone();
    let remove_status = status.clone();
    let remove_buttons = action_buttons;
    let remove_buffer = log_buffer.clone();
    let remove_entry = target_entry.clone();
    let remove_check = danger_check.clone();
    remove_btn.connect_clicked(move |_| {
        let target = remove_entry.text().trim().to_string();
        if target.is_empty() {
            append_log(
                &remove_buffer,
                "Remove container",
                Err("Container name or ID is required.".to_string()),
            );
            return;
        }

        if !remove_check.is_active() {
            append_log(
                &remove_buffer,
                "Remove container",
                Err("Enable destructive actions first.".to_string()),
            );
            return;
        }

        run_action(
            Action::Remove(target),
            remove_sender.clone(),
            &remove_status,
            &remove_buttons,
            &remove_buffer,
        );
    });

    run_action(
        Action::Refresh,
        sender,
        &status,
        &[refresh_btn, start_btn, stop_btn, remove_btn],
        &log_buffer,
    );

    window.present();
}

fn run_action(
    action: Action,
    sender: glib::Sender<WorkerMessage>,
    status: &Label,
    buttons: &[Button],
    buffer: &TextBuffer,
) {
    set_buttons_enabled(buttons, false);
    status.set_text("Status: Running...");
    append_log(
        buffer,
        "Command queued",
        Ok(action_description(&action).to_string()),
    );

    thread::spawn(move || {
        let message = process_action(action);
        let _ = sender.send(message);
    });
}

fn process_action(action: Action) -> WorkerMessage {
    match action {
        Action::Refresh => match DockerClient::list_containers() {
            Ok(containers) => WorkerMessage {
                headline: "Refresh containers".to_string(),
                details: Ok(format!("Loaded {} container(s).", containers.len())),
                containers: Some(containers),
            },
            Err(err) => WorkerMessage {
                headline: "Refresh containers".to_string(),
                details: Err(err),
                containers: None,
            },
        },
        Action::Start(target) => match DockerClient::start_container(&target) {
            Ok(out) => with_refresh("Start container", out),
            Err(err) => WorkerMessage {
                headline: format!("Start container: {target}"),
                details: Err(err),
                containers: None,
            },
        },
        Action::Stop(target) => match DockerClient::stop_container(&target) {
            Ok(out) => with_refresh("Stop container", out),
            Err(err) => WorkerMessage {
                headline: format!("Stop container: {target}"),
                details: Err(err),
                containers: None,
            },
        },
        Action::Remove(target) => match DockerClient::remove_container(&target) {
            Ok(out) => with_refresh("Remove container", out),
            Err(err) => WorkerMessage {
                headline: format!("Remove container: {target}"),
                details: Err(err),
                containers: None,
            },
        },
    }
}

fn with_refresh(headline: &str, output: String) -> WorkerMessage {
    match DockerClient::list_containers() {
        Ok(containers) => WorkerMessage {
            headline: headline.to_string(),
            details: Ok(output),
            containers: Some(containers),
        },
        Err(err) => WorkerMessage {
            headline: headline.to_string(),
            details: Err(format!("{output}\n\nRefresh failed: {err}")),
            containers: None,
        },
    }
}

fn populate_container_list(list: &ListBox, containers: &[ContainerInfo]) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    if containers.is_empty() {
        let row = ListBoxRow::new();
        let label = Label::new(Some("No containers found."));
        label.set_halign(Align::Start);
        label.add_css_class("dim-label");
        row.set_child(Some(&label));
        row.set_selectable(false);
        row.set_activatable(false);
        list.append(&row);
        return;
    }

    for container in containers {
        let row = ListBoxRow::new();
        row.set_tooltip_text(Some(&container.name));

        let content = GtkBox::new(Orientation::Vertical, 3);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        let title = Label::new(Some(&format!("{} ({})", container.name, container.id)));
        title.set_halign(Align::Start);
        title.set_xalign(0.0);
        title.add_css_class("heading");

        let status = Label::new(Some(&container.status));
        status.set_halign(Align::Start);
        status.set_xalign(0.0);
        status.add_css_class("dim-label");

        let image = Label::new(Some(&format!("Image: {}", container.image)));
        image.set_halign(Align::Start);
        image.set_xalign(0.0);
        image.add_css_class("caption");

        content.append(&title);
        content.append(&status);
        content.append(&image);
        row.set_child(Some(&content));
        list.append(&row);
    }
}

fn append_log(buffer: &TextBuffer, title: &str, details: Result<String, String>) {
    let mut iter = buffer.end_iter();
    let text = match details {
        Ok(msg) => format!("[OK] {title}\n{msg}\n\n"),
        Err(msg) => format!("[ERROR] {title}\n{msg}\n\n"),
    };
    buffer.insert(&mut iter, &text);
}

fn set_buttons_enabled(buttons: &[Button], enabled: bool) {
    buttons
        .iter()
        .for_each(|button| button.set_sensitive(enabled));
}

fn action_description(action: &Action) -> &str {
    match action {
        Action::Refresh => "Refreshing container list...",
        Action::Start(_) => "Starting container...",
        Action::Stop(_) => "Stopping container...",
        Action::Remove(_) => "Removing container...",
    }
}

fn action_button(label: &str, icon_name: &str) -> Button {
    let button = Button::new();
    let content = GtkBox::new(Orientation::Horizontal, 6);
    let icon = Image::from_icon_name(icon_name);
    let text = Label::new(Some(label));
    content.append(&icon);
    content.append(&text);
    button.set_child(Some(&content));
    button
}
