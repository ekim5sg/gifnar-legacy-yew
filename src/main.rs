use gloo::utils::window;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use wasm_bindgen::JsCast;
use js_sys;

const LS_KEY: &str = "gifnar_volunteer_log_v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    id: String,
    date: String,          // YYYY-MM-DD
    org: String,           // Houston Food Bank, etc.
    hours: f32,
    tasks: String,
    reflection: String,
    tags: String,          // comma-separated
    created_at: String,    // human readable stamp
}

fn now_stamp() -> String {
    js_sys::Date::new_0().to_string().as_string().unwrap_or_else(|| "unknown".into())
}

fn uid() -> String {
    // quick unique-ish id
    format!("{}", js_sys::Date::now())
}

fn load_entries() -> Vec<Entry> {
    let storage = window().local_storage().ok().flatten();
    let Some(storage) = storage else { return vec![] };
    let Ok(Some(raw)) = storage.get_item(LS_KEY) else { return vec![] };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_entries(entries: &[Entry]) {
    let storage = window().local_storage().ok().flatten();
    let Some(storage) = storage else { return };
    if let Ok(raw) = serde_json::to_string(entries) {
        let _ = storage.set_item(LS_KEY, &raw);
    }
}

fn download_text(filename: &str, text: &str) {
    let doc = window().document().unwrap();
    let a = doc.create_element("a").unwrap();
    a.set_attribute("download", filename).unwrap();

    // data URL
    let encoded = js_sys::encode_uri_component(text);
    let href = format!("data:text/plain;charset=utf-8,{}", encoded);
    a.set_attribute("href", &href).unwrap();

    doc.body().unwrap().append_child(&a).unwrap();
    let a_el: web_sys::HtmlElement = a.dyn_into().unwrap();
    a_el.click();
    a_el.remove();
}

#[function_component(App)]
fn app() -> Html {
    let entries = use_state(|| {
        let mut e = load_entries();
        // newest first
        e.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        e
    });

    // Form state
    let date = use_state(|| "".to_string());
    let org = use_state(|| "".to_string());
    let hours = use_state(|| "1.0".to_string());
    let tasks = use_state(|| "".to_string());
    let reflection = use_state(|| "".to_string());
    let tags = use_state(|| "".to_string());

    let on_add = {
        let entries = entries.clone();
        let date = date.clone();
        let org = org.clone();
        let hours = hours.clone();
        let tasks = tasks.clone();
        let reflection = reflection.clone();
        let tags = tags.clone();

        Callback::from(move |_| {
            let d = (*date).trim().to_string();
            let o = (*org).trim().to_string();
            let h_raw = (*hours).trim().to_string();
            let t = (*tasks).trim().to_string();
            let r = (*reflection).trim().to_string();
            let g = (*tags).trim().to_string();

            if d.is_empty() || o.is_empty() {
                window().alert_with_message("Please enter at least Date and Organization.").ok();
                return;
            }

            let h: f32 = h_raw.parse().unwrap_or(0.0);
            if h <= 0.0 {
                window().alert_with_message("Hours must be greater than 0.").ok();
                return;
            }

            let mut next = (*entries).clone();
            next.insert(0, Entry {
                id: uid(),
                date: d,
                org: o,
                hours: h,
                tasks: t,
                reflection: r,
                tags: g,
                created_at: now_stamp(),
            });

            save_entries(&next);
            entries.set(next);

            // Clear some fields (keep org if desired? We'll clear tasks/reflection/tags)
            tasks.set("".into());
            reflection.set("".into());
            tags.set("".into());
        })
    };

    let on_export_json = {
        let entries = entries.clone();
        Callback::from(move |_| {
            if let Ok(raw) = serde_json::to_string_pretty(&*entries) {
                download_text("gifnar-volunteer-log.json", &raw);
            }
        })
    };

    let on_export_csv = {
        let entries = entries.clone();
        Callback::from(move |_| {
            // CSV header
            let mut out = String::from("date,organization,hours,tasks,reflection,tags,created_at\n");
            for e in entries.iter() {
                // naive CSV escaping by wrapping in quotes and doubling quotes
                let esc = |s: &str| format!("\"{}\"", s.replace('\"', "\"\""));
                out.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    esc(&e.date),
                    esc(&e.org),
                    e.hours,
                    esc(&e.tasks),
                    esc(&e.reflection),
                    esc(&e.tags),
                    esc(&e.created_at),
                ));
            }
            download_text("gifnar-volunteer-log.csv", &out);
        })
    };

    let on_clear_all = {
        let entries = entries.clone();
        Callback::from(move |_| {
            let ok = window().confirm_with_message("Clear ALL saved entries on this device?").unwrap_or(false);
            if !ok { return; }
            save_entries(&[]);
            entries.set(vec![]);
        })
    };

    // input handlers
    let bind_input = |state: UseStateHandle<String>| {
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            state.set(input.value());
        })
    };

    let bind_textarea = |state: UseStateHandle<String>| {
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            state.set(input.value());
        })
    };

    html! {
        <div class="wrap">
          <div class="hero">
            <div>
              <h1>{"Gifnar Legacy — Volunteer Log"}</h1>
              <p class="sub">
                {"Local-first log: track hours, reflections, and export for packets/interviews. (Stored on this device.)"}
              </p>
            </div>
            <div class="actions">
              <button class="primary" onclick={on_add.clone()}>{"Add Entry"}</button>
              <button onclick={on_export_json}>{"Export JSON"}</button>
              <button onclick={on_export_csv}>{"Export CSV"}</button>
              <button onclick={on_clear_all}>{"Clear All"}</button>
            </div>
          </div>

          <div class="panel">
            <h3>{"New Entry"}</h3>
            <div class="grid">
              <label>
                <small>{"Date (YYYY-MM-DD)"}</small>
                <input value={(*date).clone()} oninput={bind_input(date.clone())} placeholder="2025-12-27" />
              </label>

              <label>
                <small>{"Organization"}</small>
                <input value={(*org).clone()} oninput={bind_input(org.clone())} placeholder="Houston Food Bank" />
              </label>

              <label>
                <small>{"Hours"}</small>
                <input value={(*hours).clone()} oninput={bind_input(hours.clone())} placeholder="2.0" />
              </label>

              <label>
                <small>{"Tags (comma-separated)"}</small>
                <input value={(*tags).clone()} oninput={bind_input(tags.clone())} placeholder="volunteering, service, leadership" />
              </label>

              <label style="grid-column: 1 / -1;">
                <small>{"Tasks / Role"}</small>
                <input value={(*tasks).clone()} oninput={bind_input(tasks.clone())} placeholder="Sorting, boxing, loading, teamwork…" />
              </label>

              <label style="grid-column: 1 / -1;">
                <small>{"Reflection"}</small>
                <textarea value={(*reflection).clone()} oninput={bind_textarea(reflection.clone())}
                  placeholder="What did you learn? Who did you serve? What would you tell future-you about this day?" />
              </label>
            </div>
            <div class="actions">
              <button class="primary" onclick={on_add}>{"Add Entry"}</button>
            </div>
          </div>

          <div class="panel">
            <h3>{format!("Saved Entries ({})", entries.len())}</h3>
            <div class="list">
              { for entries.iter().map(|e| html!{
                <div class="item" key={e.id.clone()}>
                  <div class="meta">
                    <span>{format!("Date: {}", e.date)}</span>
                    <span>{format!("Hours: {}", e.hours)}</span>
                    <span>{format!("Created: {}", e.created_at)}</span>
                  </div>
                  <div class="title">{format!("{}", e.org)}</div>
                  <hr/>
                  if !e.tasks.trim().is_empty() {
                    <div><b>{"Tasks: "}</b>{e.tasks.clone()}</div>
                  }
                  if !e.reflection.trim().is_empty() {
                    <div style="margin-top:6px;"><b>{"Reflection: "}</b>{e.reflection.clone()}</div>
                  }
                  if !e.tags.trim().is_empty() {
                    <div style="margin-top:6px;"><b>{"Tags: "}</b>{e.tags.clone()}</div>
                  }
                </div>
              }) }
            </div>
          </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
