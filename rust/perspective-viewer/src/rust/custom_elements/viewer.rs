////////////////////////////////////////////////////////////////////////////////
//
// Copyright (c) 2018, the Perspective Authors.
//
// This file is part of the Perspective library, distributed under the terms
// of the Apache License 2.0.  The full license can be found in the LICENSE
// file.

use crate::components::{Msg, PerspectiveViewer, PerspectiveViewerProps};
use crate::config::*;
use crate::custom_events::*;
use crate::dragdrop::*;
use crate::js::*;
use crate::model::*;
use crate::renderer::*;
use crate::session::Session;
use crate::theme::*;
use crate::utils::*;
use crate::*;

use js_intern::*;
use js_sys::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;
use yew::prelude::*;

struct ResizeObserverHandle {
    elem: HtmlElement,
    observer: ResizeObserver,
    _callback: Closure<dyn FnMut(js_sys::Array)>,
}

impl ResizeObserverHandle {
    fn new(
        elem: &HtmlElement,
        renderer: &Renderer,
        root: &AppHandle<PerspectiveViewer>,
    ) -> ResizeObserverHandle {
        let on_resize = root.callback(|()| Msg::Resize);
        let mut state = ResizeObserverState {
            elem: elem.clone(),
            renderer: renderer.clone(),
            width: elem.offset_width(),
            height: elem.offset_height(),
            on_resize,
        };

        let _callback = (move |xs| state.on_resize(&xs)).into_closure_mut();
        let func = _callback.as_ref().unchecked_ref::<js_sys::Function>();
        let observer = ResizeObserver::new(func);
        observer.observe(elem);
        ResizeObserverHandle {
            elem: elem.clone(),
            _callback,
            observer,
        }
    }
}

impl Drop for ResizeObserverHandle {
    fn drop(&mut self) {
        self.observer.unobserve(&self.elem);
    }
}

struct ResizeObserverState {
    elem: HtmlElement,
    renderer: Renderer,
    width: i32,
    height: i32,
    on_resize: Callback<()>,
}

impl ResizeObserverState {
    fn on_resize(&mut self, entries: &js_sys::Array) {
        let is_visible = self
            .elem
            .offset_parent()
            .map(|x| !x.is_null())
            .unwrap_or(false);

        for y in entries.iter() {
            let entry: ResizeObserverEntry = y.unchecked_into();
            let content = entry.content_rect();
            let content_width = content.width().floor() as i32;
            let content_height = content.height().floor() as i32;
            let resized = self.width != content_width || self.height != content_height;
            if resized && is_visible {
                clone!(self.on_resize, self.renderer);
                ApiFuture::spawn(async move {
                    renderer.resize().await?;
                    on_resize.emit(());
                    Ok(())
                });
            }

            self.width = content_width;
            self.height = content_height;
        }
    }
}

/// A `customElements` class which encapsulates both the `<perspective-viewer>`
/// public API, as well as the Rust component state.
///
///     ┌───────────────────────────────────────────┐
///     │ Custom Element                            │
///     │┌──────────────┐┌─────────────────────────┐│
///     ││ yew::app     ││ Model                   ││
///     ││┌────────────┐││┌─────────┐┌────────────┐││
///     │││ Components ││││ Session ││ Renderer   │││
///     ││└────────────┘│││┌───────┐││┌──────────┐│││
///     │└──────────────┘│││ Table ││││ Plugin   ││││
///     │┌──────────────┐││└───────┘││└──────────┘│││
///     ││ HtmlElement  │││┌───────┐│└────────────┘││
///     │└──────────────┘│││ View  ││┌────────────┐││
///     │                ││└───────┘││ DragDrop   │││
///     │                │└─────────┘└────────────┘││
///     │                │┌──────────────┐┌───────┐││
///     │                ││ CustomEvents ││ Theme │││
///     │                │└──────────────┘└───────┘││
///     │                └─────────────────────────┘│
///     └───────────────────────────────────────────┘
#[wasm_bindgen]
pub struct PerspectiveViewerElement {
    elem: HtmlElement,
    root: Rc<RefCell<Option<AppHandle<PerspectiveViewer>>>>,
    resize_handle: Rc<RefCell<Option<ResizeObserverHandle>>>,
    session: Session,
    renderer: Renderer,
    theme: Theme,
    _events: CustomEvents,
    _subscriptions: Rc<Subscription>,
}

derive_model!(Renderer, Session, Theme for PerspectiveViewerElement);

impl CustomElementMetadata for PerspectiveViewerElement {
    const CUSTOM_ELEMENT_NAME: &'static str = "perspective-viewer";
    const STATICS: &'static [&'static str] = ["registerPlugin", "getExprTKCommands"].as_slice();
}

#[wasm_bindgen]
impl PerspectiveViewerElement {
    #[wasm_bindgen(constructor)]
    pub fn new(elem: web_sys::HtmlElement) -> PerspectiveViewerElement {
        let init = web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open);
        let shadow_root = elem
            .attach_shadow(&init)
            .unwrap()
            .unchecked_into::<web_sys::Element>();

        // Application State
        let session = Session::default();
        let renderer = Renderer::new(&elem);
        let theme = Theme::new(&elem);

        // Create Yew App
        let props = yew::props!(PerspectiveViewerProps {
            elem: elem.clone(),
            session: session.clone(),
            renderer: renderer.clone(),
            theme: theme.clone(),
            dragdrop: DragDrop::default(),
            weak_link: WeakScope::default(),
        });

        let root = yew::Renderer::with_root_and_props(shadow_root, props).render();

        // Create callbacks
        let update_sub = session.table_updated.add_listener({
            clone!(renderer, session);
            move |_| {
                clone!(renderer, session);
                ApiFuture::spawn(async move { renderer.update(&session).await })
            }
        });

        let _events = CustomEvents::new(&elem, &session, &renderer, &theme);
        let resize_handle = ResizeObserverHandle::new(&elem, &renderer, &root);
        PerspectiveViewerElement {
            elem,
            root: Rc::new(RefCell::new(Some(root))),
            session,
            renderer,
            theme,
            resize_handle: Rc::new(RefCell::new(Some(resize_handle))),
            _events,
            _subscriptions: Rc::new(update_sub),
        }
    }

    #[wasm_bindgen(js_name = "connectedCallback")]
    pub fn connected_callback(&self) {}

    /// Loads a promise to a `JsPerspectiveTable` in this viewer.  Historially,
    /// `<perspective-viewer>` has accepted either a `Promise` or `Table` as an
    /// argument, so we preserve that behavior here with some loss of type
    /// precision.
    pub fn load(&self, table: JsValue) -> ApiFuture<()> {
        let promise = table
            .clone()
            .dyn_into::<js_sys::Promise>()
            .unwrap_or_else(|_| js_sys::Promise::resolve(&table));

        let mut config = ViewConfigUpdate::default();
        self.session
            .set_update_column_defaults(&mut config, &self.renderer.metadata());

        self.session.update_view_config(config);
        clone!(self.renderer, self.session);
        ApiFuture::new(async move {
            renderer
                .draw(async {
                    let table = JsFuture::from(promise)
                        .await?
                        .unchecked_into::<JsPerspectiveTable>();

                    session.reset_stats();
                    session.set_table(table).await?;
                    session.validate().await?.create_view().await
                })
                .await
        })
    }

    /// Delete the `View` and all associated state, rendering this
    /// `<perspective-viewer>` unusable and freeing all associated resources.
    /// Does not delete the supplied `Table` (as this is constructed by the
    /// callee).  Allowing a `<perspective-viewer>` to be garbage-collected
    /// without calling `delete()` will leak WASM memory.
    pub fn delete(&mut self) -> ApiFuture<bool> {
        clone!(self.renderer, self.session, self.root);
        ApiFuture::new(self.renderer.clone().with_lock(async move {
            renderer.delete()?;
            let result = session.delete();
            root.borrow_mut()
                .take()
                .ok_or("Already deleted!")?
                .destroy();
            Ok(result)
        }))
    }

    /// Get the underlying `View` for thie viewer.
    #[wasm_bindgen(js_name = "getView")]
    pub fn get_view(&self) -> ApiFuture<JsPerspectiveView> {
        let session = self.session.clone();
        ApiFuture::new(async move {
            Ok(session
                .get_view()
                .ok_or_else(|| js_intern!("No table set"))?
                .js_get())
        })
    }

    /// Get the underlying `Table` for this viewer.
    ///
    /// # Arguments
    /// - `wait_for_table` whether to wait for `load()` to be called, or fail
    ///   immediately if `load()` has not yet been called.
    #[wasm_bindgen(js_name = "getTable")]
    pub fn get_table(&self, wait_for_table: Option<bool>) -> ApiFuture<JsPerspectiveTable> {
        let session = self.session.clone();
        ApiFuture::new(async move {
            match session.get_table() {
                Some(table) => Ok(table),
                None if !wait_for_table.unwrap_or_default() => Err(JsValue::from("No table set")),
                None => {
                    session.table_loaded.listen_once().await.into_jserror()?;
                    session.get_table().ok_or_else(|| "No table set".into())
                }
            }
        })
    }

    pub fn flush(&self) -> ApiFuture<()> {
        clone!(self.renderer, self.session);
        ApiFuture::new(async move {
            if session.js_get_table().is_none() {
                session.table_loaded.listen_once().await.into_jserror()?;
                let _ = session
                    .js_get_table()
                    .ok_or_else(|| js_intern!("No table set"))?;
            };

            renderer.draw(async { Ok(&session) }).await
        })
    }

    /// Restores this element from a full/partial `JsPerspectiveViewConfig`.
    ///
    /// # Arguments
    /// - `update` The config to restore to, as returned by `.save()` in either
    ///   "json", "string" or "arraybuffer" format.
    pub fn restore(&self, update: JsValue) -> ApiFuture<()> {
        clone!(self.session, self.renderer, self.root, self.theme);
        ApiFuture::new(async move {
            let ViewerConfigUpdate {
                plugin,
                plugin_config,
                settings,
                theme: theme_name,
                mut view_config,
            } = ViewerConfigUpdate::decode(&update)?;

            let needs_restyle = match theme_name {
                OptionalUpdate::SetDefault => {
                    let current_name = theme.get_name().await;
                    if None != current_name {
                        theme.set_name(None).await?;
                        true
                    } else {
                        false
                    }
                }
                OptionalUpdate::Update(x) => {
                    let current_name = theme.get_name().await;
                    if current_name.is_some() && current_name.as_ref().unwrap() != &x {
                        theme.set_name(Some(&x)).await?;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            let plugin_changed = renderer.update_plugin(&plugin)?;
            if plugin_changed {
                session.set_update_column_defaults(&mut view_config, &renderer.metadata());
            }

            session.update_view_config(view_config);
            let draw_task = renderer.draw(async {
                let task = root
                    .borrow()
                    .as_ref()
                    .ok_or("Already deleted")?
                    .promise_message(move |x| Msg::ToggleSettingsComplete(settings, x));

                let result = async {
                    let plugin = renderer.get_active_plugin()?;
                    if let Some(plugin_config) = &plugin_config {
                        let js_config = JsValue::from_serde(plugin_config);
                        plugin.restore(&js_config.into_jserror()?);
                    }

                    session.validate().await?.create_view().await
                }
                .await;

                task.await.into_jserror()?;
                result
            });

            draw_task.await?;

            // TODO this should be part of the API for `draw()` above, such that
            // the plugin need not render twice when a theme is provided.
            if needs_restyle {
                let view = session.get_view().into_jserror()?;
                renderer.restyle_all(&view).await?;
            }

            Ok(())
        })
    }

    /// Save this element to serialized state object, one which can be restored
    /// via the `.restore()` method.
    ///
    /// # Arguments
    /// - `format` Supports "json" (default), "arraybuffer" or "string".
    pub fn save(&self, format: Option<String>) -> ApiFuture<JsValue> {
        let viewer_config_task = self.get_viewer_config();
        ApiFuture::new(async move {
            let format = format
                .as_ref()
                .map(|x| ViewerConfigEncoding::from_str(x))
                .transpose()?;

            let viewer_config = viewer_config_task.await?;
            viewer_config.encode(&format)
        })
    }

    /// Download this viewer's `View` or `Table` data as a `.csv` file.
    ///
    /// # Arguments
    /// - `flat` Whether to use the current `ViewConfig` to generate this data,
    ///   or use the default.
    pub fn download(&self, flat: Option<bool>) -> ApiFuture<()> {
        let session = self.session.clone();
        ApiFuture::new(async move {
            let val = session
                .csv_as_jsvalue(flat.unwrap_or_default())
                .await?
                .as_blob()?;
            download("untitled.csv", &val)
        })
    }

    /// Copy this viewer's `View` or `Table` data as CSV to the system
    /// clipboard.
    ///
    /// # Arguments
    /// - `flat` Whether to use the current `ViewConfig` to generate this data,
    ///   or use the default.
    pub fn copy(&self, flat: Option<bool>) -> ApiFuture<()> {
        let method = if flat.unwrap_or_default() {
            ExportMethod::CsvAll
        } else {
            ExportMethod::Csv
        };

        let js_task = self.export_method_to_jsvalue(method);
        let copy_task = copy_to_clipboard(js_task, MimeType::TextPlain);
        ApiFuture::new(copy_task)
    }

    /// Reset the viewer's `ViewerConfig` to the default.
    ///
    /// # Arguments
    /// - `all` Whether to clear `expressions` also.
    pub fn reset(&self, reset_expressions: Option<bool>) -> ApiFuture<()> {
        let root = self.root.clone();
        let all = reset_expressions.unwrap_or_default();
        ApiFuture::new(async move {
            let task = root
                .borrow()
                .as_ref()
                .ok_or("Already deleted")?
                .promise_message(move |x| Msg::Reset(all, Some(x)));

            task.await.into_jserror()
        })
    }

    /// Recalculate the viewer's dimensions and redraw.
    #[wasm_bindgen(js_name = "notifyResize")]
    pub fn resize(&self, force: Option<bool>) -> ApiFuture<()> {
        if !force.unwrap_or_default() && self.resize_handle.borrow().is_some() {
            let msg: JsValue = "`notifyResize(false)` called, disabling auto-size.  It can be \
                                re-enabled with `setAutoSize(true)`."
                .into();
            web_sys::console::warn_1(&msg);
            *self.resize_handle.borrow_mut() = None;
        }

        let renderer = self.renderer.clone();
        ApiFuture::new(async move { renderer.resize().await })
    }

    /// Sets the auto-size behavior of this component.  When `true`, this
    /// `<perspective-viewer>` will register a `ResizeObserver` on itself and
    /// call `resize()` whenever its own dimensions change.
    ///
    /// # Arguments
    /// - `autosize` Whether to register a `ResizeObserver` on this element or
    ///   not.
    #[wasm_bindgen(js_name = "setAutoSize")]
    pub fn set_auto_size(&mut self, autosize: bool) {
        if autosize {
            let handle = Some(ResizeObserverHandle::new(
                &self.elem,
                &self.renderer,
                self.root.borrow().as_ref().unwrap(),
            ));
            *self.resize_handle.borrow_mut() = handle;
        } else {
            *self.resize_handle.borrow_mut() = None;
        }
    }

    /// Get this viewer's edit port for the currently loaded `Table`.
    #[wasm_bindgen(js_name = "getEditPort")]
    pub fn get_edit_port(&self) -> Result<f64, JsValue> {
        self.session
            .metadata()
            .get_edit_port()
            .ok_or_else(|| "No `Table` loaded".into())
    }

    /// Restyle all plugins from current document.
    #[wasm_bindgen(js_name = "restyleElement")]
    pub fn restyle_element(&self) -> ApiFuture<JsValue> {
        clone!(self.renderer, self.session);
        ApiFuture::new(async move {
            let view = session.get_view().into_jserror()?;
            renderer.restyle_all(&view).await
        })
    }

    /// Set the available theme names available in the status bar UI.
    #[wasm_bindgen(js_name = "resetThemes")]
    pub fn reset_themes(&self, themes: Option<Box<[JsValue]>>) -> ApiFuture<JsValue> {
        clone!(self.renderer, self.session, self.theme);
        ApiFuture::new(async move {
            let themes: Option<Vec<String>> = themes
                .unwrap_or_default()
                .iter()
                .map(|x| x.as_string())
                .collect();

            let theme_name = theme.get_name().await;
            theme.reset(themes).await;
            let reset_theme = theme
                .get_themes()
                .await?
                .iter()
                .find(|y| theme_name.as_ref() == Some(y))
                .cloned();

            theme.set_name(reset_theme.as_deref()).await?;
            let view = session.get_view().into_jserror()?;
            renderer.restyle_all(&view).await
        })
    }

    /// Determines the render throttling behavior. Can be an integer, for
    /// millisecond window to throttle render event; or, if `None`, adaptive
    /// throttling will be calculated from the measured render time of the
    /// last 5 frames.
    ///
    /// # Examples
    /// // Only draws at most 1 frame/sec.
    /// viewer.js_set_throttle(Some(1000_f64));
    ///
    /// # Arguments
    /// - `throttle` The throttle rate - milliseconds (f64), or `None` for
    ///   adaptive throttling.
    #[wasm_bindgen(js_name = "setThrottle")]
    pub fn set_throttle(&mut self, val: Option<f64>) {
        self.renderer.set_throttle(val);
    }

    /// Toggle (or force) the config panel open/closed.
    ///
    /// # Arguments
    /// - `force` Force the state of the panel open or closed, or `None` to
    ///   toggle.
    #[wasm_bindgen(js_name = "toggleConfig")]
    pub fn toggle_config(&self, force: Option<bool>) -> ApiFuture<JsValue> {
        let root = self.root.clone();
        ApiFuture::new(async move {
            let force = force.map(SettingsUpdate::Update);
            let task = root
                .borrow()
                .as_ref()
                .into_jserror()?
                .promise_message(|x| Msg::ToggleSettingsInit(force, Some(x)));

            task.await.map_err(|_| JsValue::from("Cancelled"))?
        })
    }

    /// Get an `Array` of all of the plugin custom elements registered for this
    /// element. This may not include plugins which called
    /// `registerPlugin()` after the host has rendered for the first time.
    #[wasm_bindgen(js_name = "getAllPlugins")]
    pub fn get_all_plugins(&self) -> Array {
        self.renderer.get_all_plugins().iter().collect::<Array>()
    }

    /// Gets a plugin Custom Element with the `name` field, or get the active
    /// plugin if no `name` is provided.
    ///
    /// # Arguments
    /// - `name` The `name` property of a perspective plugin Custom Element, or
    ///   `None` for the active plugin's Custom Element.
    #[wasm_bindgen(js_name = "getPlugin")]
    pub fn get_plugin(&self, name: Option<String>) -> Result<JsPerspectiveViewerPlugin, JsValue> {
        match name {
            None => self.renderer.get_active_plugin(),
            Some(name) => self.renderer.get_plugin(&name),
        }
    }

    /// Internal Only.
    ///
    /// Get this custom element model's raw pointer.
    #[wasm_bindgen(js_name = "unsafeGetModel")]
    pub fn unsafe_get_model(&self) -> *const PerspectiveViewerElement {
        std::ptr::addr_of!(*self)
    }
}
