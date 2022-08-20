////////////////////////////////////////////////////////////////////////////////
//
// Copyright (c) 2018, the Perspective Authors.
//
// This file is part of the Perspective library, distributed under the terms
// of the Apache License 2.0.  The full license can be found in the LICENSE
// file.

use super::color_selector::*;
use super::containers::radio_list::RadioList;
use super::containers::radio_list_item::RadioListItem;
use super::modal::{ModalLink, SetModalLink};
use crate::config::*;
use crate::utils::WeakScope;
use crate::*;
use wasm_bindgen::*;
use web_sys::*;
use yew::prelude::*;
use yew::*;

pub static CSS: &str = include_str!("../../../build/css/column-style.css");

pub enum StringColumnStyleMsg {
    Reset(StringColumnStyleConfig),
    FormatEnabled(bool),
    FormatChanged(FormatMode),
    ColorModeEnabled(bool),
    ColorModeChanged(StringColorMode),
    ColorChanged(String),
}

#[derive(Properties)]
pub struct StringColumnStyleProps {
    #[prop_or_default]
    pub config: StringColumnStyleConfig,

    #[prop_or_default]
    pub default_config: StringColumnStyleDefaultConfig,

    #[prop_or_default]
    pub on_change: Callback<StringColumnStyleConfig>,

    #[prop_or_default]
    weak_link: WeakScope<StringColumnStyle>,
}

impl ModalLink<StringColumnStyle> for StringColumnStyleProps {
    fn weak_link(&self) -> &'_ WeakScope<StringColumnStyle> {
        &self.weak_link
    }
}

impl PartialEq for StringColumnStyleProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

/// The `ColumnStyle` component stores its UI state privately in its own struct,
/// rather than its props (which has two version of this data itself, the
/// JSON serializable config record and the defaults record).
pub struct StringColumnStyle {
    config: StringColumnStyleConfig,
}

impl StringColumnStyle {
    /// When this config has changed, we must signal the wrapper element.
    fn dispatch_config(&self, ctx: &Context<Self>) {
        ctx.props().on_change.emit(self.config.clone());
    }

    /// Generate a color selector component for a specific `StringColorMode`
    /// variant.
    fn color_select_row(&self, ctx: &Context<Self>, mode: &StringColorMode, title: &str) -> Html {
        let on_color = ctx.link().callback(StringColumnStyleMsg::ColorChanged);
        let color = self
            .config
            .color
            .clone()
            .unwrap_or_else(|| ctx.props().default_config.color.to_owned());

        let color_props = props!(ColorProps { color, on_color });
        if let Some(x) = &self.config.string_color_mode && x == mode {
            html_template! {
                <span class="row">{ title }</span>
                <div class="row inner_section">
                    <ColorSelector ..color_props />
                </div>
            }
        } else {
            html! {
                <span class="row">{ title }</span>
            }
        }
    }
}

impl Component for StringColumnStyle {
    type Message = StringColumnStyleMsg;
    type Properties = StringColumnStyleProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.set_modal_link();
        StringColumnStyle {
            config: ctx.props().config.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StringColumnStyleMsg::Reset(config) => {
                self.config = config;
                true
            }
            StringColumnStyleMsg::FormatEnabled(val) => {
                self.config.format = if val {
                    Some(FormatMode::default())
                } else {
                    None
                };

                self.dispatch_config(ctx);
                true
            }
            StringColumnStyleMsg::FormatChanged(val) => {
                self.config.format = Some(val);
                self.dispatch_config(ctx);
                true
            }
            StringColumnStyleMsg::ColorModeEnabled(enabled) => {
                if enabled {
                    self.config.string_color_mode = Some(StringColorMode::default());
                } else {
                    self.config.string_color_mode = None;
                    self.config.color = None;
                }

                self.dispatch_config(ctx);
                true
            }
            StringColumnStyleMsg::ColorModeChanged(mode) => {
                self.config.string_color_mode = Some(mode);
                self.dispatch_config(ctx);
                true
            }
            StringColumnStyleMsg::ColorChanged(color) => {
                self.config.color = Some(color);
                self.dispatch_config(ctx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let format_enabled_oninput = ctx.link().callback(move |event: InputEvent| {
            let input = event
                .target()
                .unwrap()
                .unchecked_into::<web_sys::HtmlInputElement>();
            StringColumnStyleMsg::FormatEnabled(input.checked())
        });

        let format_mode_selected = self.config.format.unwrap_or_default();
        let format_mode_changed = ctx.link().callback(StringColumnStyleMsg::FormatChanged);
        let color_enabled_oninput = ctx.link().callback(move |event: InputEvent| {
            let input = event
                .target()
                .unwrap()
                .unchecked_into::<web_sys::HtmlInputElement>();
            StringColumnStyleMsg::ColorModeEnabled(input.checked())
        });

        let selected_color_mode = self.config.string_color_mode.unwrap_or_default();
        let color_mode_changed = ctx.link().callback(StringColumnStyleMsg::ColorModeChanged);

        let series_controls = self.color_select_row(ctx, &StringColorMode::Series, "Series");
        let foreground_controls =
            self.color_select_row(ctx, &StringColorMode::Foreground, "Foreground");

        let background_controls =
            self.color_select_row(ctx, &StringColorMode::Background, "Background");

        html_template! {
            <style>
                { &CSS }
            </style>
            <div id="column-style-container">
                <div class="column-style-label">
                    <label class="indent">{ "Format" }</label>
                </div>
                <div class="section">
                    <input
                        type="checkbox"
                        oninput={ format_enabled_oninput }
                        checked={ self.config.format.is_some() } />

                    <RadioList<FormatMode>
                        class="indent"
                        disabled={ self.config.format.is_none() }
                        selected={ format_mode_selected }
                        on_change={ format_mode_changed } >

                        <RadioListItem<FormatMode>
                            value={ FormatMode::Bold }>
                            <span>{ "Bold" }</span>
                        </RadioListItem<FormatMode>>
                        <RadioListItem<FormatMode>
                            value={ FormatMode::Italics }>
                            <span>{ "Italics" }</span>
                        </RadioListItem<FormatMode>>
                        <RadioListItem<FormatMode>
                            value={ FormatMode::Link }>
                            <span>{ "Link" }</span>
                        </RadioListItem<FormatMode>>
                    </RadioList<FormatMode>>
                </div>
                <div class="column-style-label">
                    <label class="indent">{ "Color" }</label>
                </div>
                <div class="section">
                    <input
                        type="checkbox"
                        oninput={ color_enabled_oninput }
                        checked={ self.config.string_color_mode.is_some() } />

                    <RadioList<StringColorMode>
                        class="indent"
                        name="color-radio-list"
                        disabled={ self.config.string_color_mode.is_none() }
                        selected={ selected_color_mode }
                        on_change={ color_mode_changed } >

                        <RadioListItem<StringColorMode>
                            value={ StringColorMode::Foreground }>
                            { foreground_controls }
                        </RadioListItem<StringColorMode>>
                        <RadioListItem<StringColorMode>
                            value={ StringColorMode::Background }>
                            { background_controls }
                        </RadioListItem<StringColorMode>>
                        <RadioListItem<StringColorMode>
                            value={ StringColorMode::Series }>
                            { series_controls }
                        </RadioListItem<StringColorMode>>
                    </RadioList<StringColorMode>>
                </div>
            </div>
        }
    }
}
