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

/// [Use incremental TT munching](https://veykril.github.io/tlborm/decl-macros/patterns/tt-muncher.html)
#[macro_export]
macro_rules! box_start_with_component {
  (
    in:                     $arg_surface : expr,                 // Eg: in: tw_surface,
    id:                     $arg_id : expr,                      // Eg: "foo",
    dir:                    $arg_dir : expr,                     // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr,  // Eg: (50, 100).try_into()?,
    styles:                 [$($args:tt)*],                      // Eg: [ "style1" , "style2" ]
    render:                 {$($tail:tt)*}                       // Eg: render! args
  ) => {
    box_start! {
      in:       $arg_surface,
      id:       $arg_id,
      dir:      $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      styles:   [$($args)*]
    };

    render! {
      in:           $arg_surface,
      component_id: $arg_id,
      $($tail)*
    };

    $arg_surface.box_end()?;
  };
}

/// `self` has to be passed into `$arg_runnable` because this macro has a `let` statement that
/// requires it to have a block. And in the block generated by the macro, `self` is not available
/// from the calling scope.
#[macro_export]
macro_rules! box_start_with_runnable {
  (
    in:                     $arg_surface        : expr,           // Eg: in: tw_surface,
    runnable:               $arg_runnable       : expr,           // Eg: runnable: two_col_layout,
    id:                     $arg_id             : expr,           // Eg: "foo",
    dir:                    $arg_dir            : expr,           // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr,   // Eg: (50, 100).try_into()?,
    styles:                 [$($args_styles:tt)*],                // Eg: [ "style1" , "style2" ]
    state:                  $arg_state          : expr,           // Eg: state,
    shared_store:           $arg_shared_store   : expr,           // Eg: shared_store
    shared_tw_data:         $arg_shared_tw_data : expr            // Eg: shared_tw_data
  ) => {
    box_start! {
      in:       $arg_surface,
      id:       $arg_id,
      dir:      $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      styles:   [$($args_styles)*]
    };

    $arg_runnable
      .run_on_surface($arg_surface, $arg_state, $arg_shared_store, $arg_shared_tw_data)
      .await?;

    $arg_surface.box_end()?;
  };
}

/// `self` has to be passed into `$arg_runnable` because this macro has a `let` statement that
/// requires it to have a block. And in the block generated by the macro, `self` is not available
/// from the calling scope.
#[macro_export]
macro_rules! surface_start_with_runnable {
  (
    runnable:       $arg_runnable       : expr, // Eg: runnable: two_col_layout,
    stylesheet:     $arg_stylesheet     : expr, // Eg: stylesheet,
    pos:            $arg_pos            : expr, // Eg: (0, 0).into(),
    size:           $arg_size           : expr, // Eg: (50, 100).into(),
    state:          $arg_state          : expr, // Eg: state,
    shared_store:   $arg_shared_store   : expr, // Eg: shared_store
    shared_tw_data: $arg_shared_tw_data : expr  // Eg: shared_tw_data
  ) => {{
    let mut surface = Surface {
      stylesheet: $arg_stylesheet,
      ..Surface::default()
    };

    surface.surface_start(SurfaceProps {
      pos: $arg_pos,
      size: $arg_size,
    })?;

    $arg_runnable
      .run_on_surface(
        &mut surface,
        $arg_state,
        $arg_shared_store,
        $arg_shared_tw_data,
      )
      .await?;

    surface.surface_end()?;

    surface
  }};
}

#[macro_export]
macro_rules! box_start {
  (
    in:                     $arg_surface : expr,                // Eg: in: tw_surface,
    id:                     $arg_id : expr,                     // Eg: "foo",
    dir:                    $arg_dir : expr,                    // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr, // Eg: (50, 100).try_into()?,
    styles:                 [$($args:tt)*]                      // Eg: [ "style1" , "style2" ]
  ) => {
    $arg_surface.box_start(box_props! {
      id:                     $arg_id,
      dir:                    $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      maybe_styles:           get_styles! { from: $arg_surface.stylesheet, [$($args)*] }
    })?
  };
}

#[macro_export]
macro_rules! box_props {
  (
    id:                     $arg_id : expr,                     // Eg: "foo",
    dir:                    $arg_dir : expr,                    // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr, // Eg: (50, 100).try_into()?,
    maybe_styles:           $arg_styles: expr                   // Eg: get_styles!
                                                                //     { from: stylesheet,
                                                                //     ["style1", "style2"] };
  ) => {
    FlexBoxProps {
      id: $arg_id.to_string(),
      dir: $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      maybe_styles: $arg_styles,
    }
  };
  (
    id:                     $arg_id : expr,                     // Eg: "foo",
    dir:                    $arg_dir : expr,                    // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr, // Eg: (50, 100).try_into()?,
    maybe_styles:           [$($args:tt)*]                      // Eg: [style!{...} , style!{...}]
  ) => {
    FlexBoxProps {
      id: $arg_id.to_string(),
      dir: $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      maybe_styles: Some(vec![$($args)*]),
    }
  };
  (
    id:       $arg_id : expr,                                   // Eg: "foo",
    dir:      $arg_dir : expr,                                  // Eg: Direction::Horizontal,
    requested_size_percent: $arg_requested_size_percent : expr, // Eg: (50, 100).try_into()?,
  ) => {
    FlexBoxProps {
      id: $arg_id.to_string(),
      dir: $arg_dir,
      requested_size_percent: $arg_requested_size_percent,
      maybe_styles: None,
    }
  };
}
