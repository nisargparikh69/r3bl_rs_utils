/*
 *   Copyright (c) 2022 R3BL LLC
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use async_trait::async_trait;
use r3bl_rs_utils_core::*;
use tokio::sync::RwLock;

use crate::*;

/// This is a shim which allows the reusable [DialogEngine] to be used in the context of [Component]
/// and [r3bl_redux::Store]. The main methods here simply pass thru all their arguments to the
/// [DialogEngine].
#[derive(Clone, Default)]
pub struct DialogComponent<S, A>
where
    S: Debug + Default + Clone + PartialEq + Sync + Send,
    A: Debug + Default + Clone + Sync + Send,
{
    pub id: FlexBoxId,
    pub dialog_engine: DialogEngine,
    /// Make sure to dispatch actions to handle the user's dialog choice [DialogChoice].
    pub on_dialog_press_handler: Option<OnDialogPressFn<S, A>>,
    /// Make sure to dispatch an action to update the dialog buffer's editor buffer.
    pub on_dialog_editor_change_handler: Option<OnDialogEditorChangeFn<S, A>>,
}

#[async_trait]
impl<S, A> Component<S, A> for DialogComponent<S, A>
where
    S: HasDialogBuffers + Default + Clone + PartialEq + Debug + Sync + Send,
    A: Debug + Default + Clone + Sync + Send,
{
    fn reset(&mut self) { self.dialog_engine.reset(); }

    fn get_id(&self) -> FlexBoxId { self.id }

    /// This shim simply calls [DialogEngineApi::render_engine](DialogEngineApi::render_engine) w/
    /// all the necessary arguments:
    /// - Global scope: [r3bl_redux::SharedStore], [SharedGlobalData].
    /// - App scope: `S`, [ComponentRegistry<S, A>].
    /// - User input (from [main_event_loop]): [InputEvent].
    ///
    /// Note:
    /// 1. The 3rd argument `_current_box` [FlexBox] is ignored since the dialog component
    ///    breaks out of whatever box the layout places it in, and ends up painting itself over
    ///    the entire screen.
    /// 2. However, `SurfaceBounds` is saved for later use. And it is used to restrict where the
    ///    dialog can be placed on the screen.
    async fn render(
        &mut self,
        args: ComponentScopeArgs<'_, S, A>,
        _current_box: &FlexBox,        /* Ignore this. */
        surface_bounds: SurfaceBounds, /* Save this. */
    ) -> CommonResult<RenderPipeline> {
        self.dialog_engine.maybe_surface_bounds = Some(surface_bounds);

        let ComponentScopeArgs {
            state,
            shared_store,
            shared_global_data,
            component_registry,
            window_size,
        } = args;

        let dialog_buffer: Cow<DialogBuffer> =
            if let Some(it) = state.get_dialog_buffer(self.get_id()) {
                Cow::Borrowed(it)
            } else {
                Cow::Owned(DialogBuffer::new_empty())
            };

        let dialog_engine_args = {
            DialogEngineArgs {
                shared_global_data,
                shared_store,
                state,
                component_registry,
                self_id: self.get_id(),
                dialog_engine: &mut self.dialog_engine,
                dialog_buffer: &dialog_buffer,
                window_size,
            }
        };

        DialogEngineApi::render_engine(dialog_engine_args).await
    }

    /// This shim simply calls [DialogEngineApi::apply_event](DialogEngineApi::apply_event) w/ all
    /// the necessary arguments:
    /// - Global scope: [r3bl_redux::SharedStore], [SharedGlobalData].
    /// - App scope: `S`, [ComponentRegistry<S, A>].
    /// - User input (from [main_event_loop]): [InputEvent].
    ///
    /// Usually a component must have focus in order for the [App] to
    /// [route_event_to_focused_component](ComponentRegistry::route_event_to_focused_component) in
    /// the first place.
    async fn handle_event(
        &mut self,
        args: ComponentScopeArgs<'_, S, A>,
        input_event: &InputEvent,
    ) -> CommonResult<EventPropagation> {
        let ComponentScopeArgs {
            state,
            shared_store,
            shared_global_data,
            component_registry,
            window_size,
        } = args;

        let dialog_buffer: Cow<DialogBuffer> =
            if let Some(it) = state.get_dialog_buffer(self.get_id()) {
                Cow::Borrowed(it)
            } else {
                Cow::Owned(DialogBuffer::new_empty())
            };

        let dialog_engine_args = {
            DialogEngineArgs {
                shared_global_data,
                shared_store,
                state,
                component_registry,
                self_id: self.get_id(),
                dialog_engine: &mut self.dialog_engine,
                dialog_buffer: &dialog_buffer,
                window_size,
            }
        };

        match DialogEngineApi::apply_event(dialog_engine_args, input_event).await? {
            // Handler user's choice.
            DialogEngineApplyResponse::DialogChoice(dialog_choice) => {
                component_registry.has_focus.reset_modal_id();

                call_if_true!(DEBUG_TUI_MOD, {
                    let msg = format!(
                        "🐝 restore focus to non modal: {:?}",
                        component_registry.has_focus
                    );
                    log_debug(msg);
                });

                // Run the handler (if any) w/ `dialog_choice`.
                if let Some(it) = &self.on_dialog_press_handler {
                    it(dialog_choice, shared_store);
                };

                // Trigger re-render, now that focus has been restored to non-modal component.
                Ok(EventPropagation::ConsumedRender)
            }

            // Handler user input that has updated the dialog_buffer.editor_buffer.
            DialogEngineApplyResponse::UpdateEditorBuffer(new_editor_buffer) => {
                // Run the handler (if any) w/ `new_editor_buffer`.
                if let Some(it) = &self.on_dialog_editor_change_handler {
                    it(new_editor_buffer, shared_store);
                };

                // The handler should dispatch action to change state since dialog_buffer.editor_buffer is
                // updated.
                Ok(EventPropagation::Consumed)
            }

            // Handle user input that has updated the results panel.
            DialogEngineApplyResponse::SelectScrollResultsPanel => {
                Ok(EventPropagation::ConsumedRender)
            }

            // All else.
            _ => Ok(EventPropagation::Propagate),
        }
    }
}

impl<S, A> DialogComponent<S, A>
where
    S: Debug + Default + Clone + PartialEq + Sync + Send,
    A: Debug + Default + Clone + Sync + Send,
{
    /// The on_dialog_press_handler is a lambda that is called if the user presses enter or escape.
    /// Typically this results in a Redux action being created and then dispatched to the given
    /// store.
    pub fn new(
        id: FlexBoxId,
        dialog_options: DialogEngineConfigOptions,
        editor_options: EditorEngineConfig,
        on_dialog_press_handler: OnDialogPressFn<S, A>,
        on_dialog_editor_change_handler: OnDialogEditorChangeFn<S, A>,
    ) -> Self {
        Self {
            dialog_engine: DialogEngine::new(dialog_options, editor_options),
            id,
            on_dialog_press_handler: Some(on_dialog_press_handler),
            on_dialog_editor_change_handler: Some(on_dialog_editor_change_handler),
        }
    }

    pub fn new_shared(
        id: FlexBoxId,
        dialog_options: DialogEngineConfigOptions,
        editor_options: EditorEngineConfig,
        on_dialog_press_handler: OnDialogPressFn<S, A>,
        on_dialog_editor_change_handler: OnDialogEditorChangeFn<S, A>,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(DialogComponent::new(
            id,
            dialog_options,
            editor_options,
            on_dialog_press_handler,
            on_dialog_editor_change_handler,
        )))
    }
}
