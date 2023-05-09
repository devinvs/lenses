# User Manual

`lenses` is a simulation engine for tracing light rays through lenses. It takes
a description of a scene with multiple lenses and lights and displays an
interactive environment to inspect and observe the effects of the lenses on the
light.

Scenes are defined by creating a yaml file that follows this syntax:

```yaml
lenses:
  - radius: 0.5          # Radius of the lens
    left: !Flat          # Flat
    right: !Convex 0.2   # Convex radius, can also specify !Concave
    pos: [0.0, 0.0, 0.0] # Position of the lens

lights:
    - !Laser [
        [2.0, 0.0, 0.0],    # origin
        [-1.0, 0.0, 0.0]    # direction
      ]
    - !Point [2.0, 1.0, 0.0] # origin
```

A Laser shoots a single ray of light in a single direction while a point light
shoots 1000 rays in random directions.

To run the simulation for a file `scene.yaml` and view the output:

```sh
cargo run --release -- scene.yaml
```

This will launch an interactive window where you can scroll to zoom in and out
and click and drag to rotate the scene.

____________

Author: Devin Vander Stelt <devin@vstelt.dev>
