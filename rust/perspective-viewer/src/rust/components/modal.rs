////////////////////////////////////////////////////////////////////////////////
//
// Copyright (c) 2018, the Perspective Authors.
//
// This file is part of the Perspective library, distributed under the terms
// of the Apache License 2.0.  The full license can be found in the LICENSE
// file.

use crate::utils::WeakScope;
use crate::*;

use derivative::Derivative;
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use yew::{prelude::*, virtual_dom::VChild};

#[derive(Clone, Default, Eq, PartialEq)]
pub struct ModalOrientation(Rc<Cell<bool>>);

impl From<ModalOrientation> for bool {
    fn from(x: ModalOrientation) -> Self {
        x.0.get()
    }
}

pub trait ModalLink<T>
where
    T: Component,
{
    fn weak_link(&self) -> &'_ WeakScope<T>;
}

pub trait SetModalLink {
    fn set_modal_link(&self);
}

impl<T> SetModalLink for &Context<T>
where
    T: Component,
    T::Properties: ModalLink<T>,
{
    fn set_modal_link(&self) {
        *self.props().weak_link().borrow_mut() = Some(self.link().clone());
    }
}

pub enum ModalMsg<T>
where
    T: Component,
{
    SetPos {
        top: i32,
        left: i32,
        visible: bool,
        rev_vert: bool,
    },
    SubMsg(T::Message),
}

#[derive(Properties, Derivative)]
#[derivative(PartialEq(bound = ""))]
pub struct ModalProps<T>
where
    T: Component,
    T::Properties: ModalLink<T>,
{
    pub child: Option<VChild<T>>,
}

#[derive(Default)]
pub struct Modal<T> {
    css: String,
    rev_vert: ModalOrientation,
    _comp: PhantomData<T>,
}

impl<T> Component for Modal<T>
where
    T: Component,
    T::Properties: ModalLink<T>,
{
    type Message = ModalMsg<T>;
    type Properties = ModalProps<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Modal {
            css: ":host{{top:0px;left:0px;opacity:0}}".to_owned(),
            rev_vert: Default::default(),
            _comp: PhantomData::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: ModalMsg<T>) -> bool {
        match msg {
            ModalMsg::SetPos {
                top,
                left,
                visible,
                rev_vert,
            } => {
                let opacity = if visible { "" } else { ";opacity:0" };
                self.css = format!(":host{{top:{}px;left:{}px{}}}", top, left, opacity);
                self.rev_vert.0.set(rev_vert);
                true
            }
            ModalMsg::SubMsg(msg) => {
                if let Some(child) = &ctx.props().child {
                    if let Some(link) = child.props.weak_link().borrow().as_ref() {
                        link.send_message(msg);
                    }
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let child = ctx
            .props()
            .child
            .clone()
            .map(Html::from)
            .unwrap_or_default();

        html_template! {
            <style>
                { self.css.to_owned() }
            </style>
            <ContextProvider<ModalOrientation> context={ self.rev_vert.clone() }>
                {
                    child
                }
            </ContextProvider<ModalOrientation>>
        }
    }
}
