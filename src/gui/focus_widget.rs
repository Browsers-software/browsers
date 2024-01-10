use druid::{
    BoxConstraints, Command, Env, Event, EventCtx, KbKey, KeyEvent, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Selector, Size, Target, UpdateCtx, Widget, WidgetId,
};
use tracing::debug;

pub trait FocusData {
    fn has_autofocus(&self) -> bool;
}

pub const FOCUS_WIDGET_SET_FOCUS_ON_HOVER: Selector<WidgetId> =
    Selector::new("focus_widget.set_focus");

pub struct FocusWidget<S: druid::Data + FocusData, W> {
    inner: W,
    paint_fn_on_focus: fn(ctx: &mut PaintCtx, data: &S, env: &Env),
    lifecycle_fn: fn(ctx: &mut LifeCycleCtx, data: &S, env: &Env),
}

impl<S: druid::Data + FocusData, W> FocusWidget<S, W> {}

impl<S: druid::Data + FocusData, W> FocusWidget<S, W> {
    pub fn new(
        inner: W,
        paint_fn_on_focus: fn(ctx: &mut PaintCtx, data: &S, env: &Env),
        lifecycle_fn: fn(ctx: &mut LifeCycleCtx, data: &S, env: &Env),
    ) -> FocusWidget<S, W> {
        FocusWidget {
            inner,
            paint_fn_on_focus,
            lifecycle_fn,
        }
    }
}

impl<S: druid::Data + FocusData, W: Widget<S>> Widget<S> for FocusWidget<S, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut S, env: &Env) {
        match event {
            // on mouse hover request focus
            Event::Command(cmd) if cmd.is(FOCUS_WIDGET_SET_FOCUS_ON_HOVER) => {
                //let widget_id = cmd.get_unchecked(FOCUS_WIDGET_SET_FOCUS_ON_HOVER);
                //info!(
                //    "received FOCUS_WIDGET_SET_FOCUS to widget_id: {:?}",
                //    widget_id
                //);
                ctx.request_focus();
                ctx.request_paint();
                ctx.set_handled();
                ctx.request_update();
            }
            Event::WindowConnected => {
                if data.has_autofocus() {
                    // ask for focus on launch
                    ctx.request_focus();
                }
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::Tab,
                mods,
                ..
            }) => {
                if mods.shift() {
                    debug!("Shift+Tab PRESSED");
                    ctx.focus_prev();
                } else {
                    debug!("Tab PRESSED");
                    ctx.focus_next();
                };

                ctx.request_paint();
                ctx.set_handled();
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::ArrowDown,
                ..
            }) => {
                debug!("ArrowDown PRESSED");

                ctx.focus_next();
                ctx.request_paint();
                ctx.set_handled();
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::ArrowUp,
                ..
            }) => {
                debug!("ArrowUp PRESSED");

                ctx.focus_prev();
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }

        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &S, env: &Env) {
        match event {
            LifeCycle::BuildFocusChain => {
                // widget which can be hovered with a mouse,
                // can also be focused with keyboard navigation
                ctx.register_for_focus();
            }
            LifeCycle::FocusChanged(to_focused) => {
                if *to_focused {
                    // enable scrolling once getting edge cases right
                    // (sometimes too eager to scroll top/bottom item)
                    if !ctx.is_hot() {
                        ctx.scroll_to_view();
                    }
                    (self.lifecycle_fn)(ctx, data, env);
                }
                ctx.request_paint();
            }
            LifeCycle::HotChanged(to_hot) => {
                if *to_hot && !ctx.has_focus() {
                    // when mouse starts "hovering" this item, let's also request focus,
                    // because we consider keyboard navigation and mouse hover the same here
                    let cmd = Command::new(
                        FOCUS_WIDGET_SET_FOCUS_ON_HOVER,
                        ctx.widget_id(),
                        Target::Widget(ctx.widget_id()),
                    );
                    ctx.submit_command(cmd);
                    //ctx.request_paint();
                }
            }
            _ => {}
        }
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &S, data: &S, env: &Env) {
        /*if old_data.glow_hot != data.glow_hot {
            ctx.request_paint();
        }*/
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &S, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &S, env: &Env) {
        if ctx.has_focus() {
            (self.paint_fn_on_focus)(ctx, data, env);
        }
        self.inner.paint(ctx, data, env);
    }
}
