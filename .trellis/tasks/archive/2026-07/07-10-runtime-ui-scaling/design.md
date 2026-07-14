# Runtime UI Scaling Design

## Scope and Boundaries

The frontend owns this feature. It changes the Locus renderer's CSS scale only;
it does not change native window dimensions, operating-system display scaling,
or the UI rendered by a connected Unity project.

`src/composables/useDisplaySettings.ts` remains the single source of truth.
It will own the scale type, defaults, normalization, persistence, and the
function that applies the scale to the current document. `src/App.vue` will
initialize the setting synchronously with the existing theme and font setup,
so every renderer instance applies the saved scale before rendering its view.

## State Contract

Add `uiScale` to `DisplaySettings` as an integer percentage. Valid values are
50 through 300 inclusive, in increments of 10, with a default of 100.

The display-settings loader must normalize legacy, malformed, fractional, and
out-of-range stored values. The dedicated scale setter must also normalize
before saving and applying the result. This prevents a manually edited browser
storage value from making a renderer unusable.

## Rendering Behavior

The scale application function sets a single document-root CSS zoom value from
the normalized percentage. CSS zoom is chosen because it proportionally scales
both fixed-pixel and relative-pixel Locus styles while preserving layout flow;
changing root font size would leave much of the existing pixel-based UI
unscaled. Applying it at the shared application root covers main, standalone,
and Unity-embedded Locus windows that all boot through `App.vue`.

## Settings UI

Add a Display-settings section with:

- a range input with `min=50`, `max=300`, and `step=10`;
- decrement and increment icon buttons for precise one-step adjustment;
- the current percentage as the accessible label/value; and
- a reset icon button that sets the value to 100% and is disabled when already
  at the default.

The range input updates the scale immediately. New localized strings are added
to both English and Chinese catalogs. Controls use the project icon and button
conventions, including an accessible tooltip/label for the reset action.

## Compatibility and Rollback

Existing display-settings storage uses a merge-with-defaults load path, so
adding `uiScale` is backward compatible. Removing the feature later simply
leaves an ignored storage field; no migration is needed. Reverting the frontend
files restores the former 100% rendering behavior.
