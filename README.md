
# fastlem

![terrain](sample.webp)

[![Crates.io](https://img.shields.io/crates/v/fastlem)](https://crates.io/crates/fastlem)
[![Documentation](https://docs.rs/fastlem/badge.svg)](https://docs.rs/fastlem)

**fastlem** [/ˈfæstlɛm/] is a rust library that provides methods for simulating landscape evolution processes to create realistic terrain data with plausible relief.

The **[fastlem web simulator](https://fastlem.peruki.dev/)** is available to try out manually creating virtual terrain with fastlem.
![fastlemweb2](https://github.com/TadaTeruki/fastlem/assets/69315285/1a98d63d-853d-4e76-bba5-b399f4772bf2)

## Installation


> [!WARNING]
> This project is currently under development. During `0.1.*` the interface may change a lot.
> 
> It is recommended to specify the version in detail.

```
[dependencies]
fastlem = "0.1.2"
```

## About the simulation

fastlem follows the **Salève** model[1], which is an analytical model for simulating landscape evolution in 2D. The algorithm for creating the drainage network follows the method of Guillaume Cordonnier, Jean Braun, Marie-Paule Cani, Bedrich Benes, Eric Galin, et al[2].

### Build a graph

Terrain data is represented as a planar graph. This graph structure is used to compute all topographic simulations for terrain generation.

For fastlem with 2D model, `TerrainModel2DBulider` is required to construct the graph from given sites as `TerrainModel2D` by computing a Delaunay triangulation. You can optionally use `relaxate_sites` to move the sites to approximately equal positions using Lloyd's algorithm.

```rust
let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max) // generate sites randomly
    .relaxate_sites(1) // relaxate sites using Lloyd's algorithm
    .unwrap()
    .build() // build
    .unwrap();
```

### Create terrain generator

Before starting terrain generation, `TerrainGenerator` is required to use the planer graph and assign the topographic parameters to each site.

```rust 
let terrain_generator = TerrainGenerator::default()
    .set_model(model)
    .set_parameters(
        (0..num)
            .map(|_| TopographicalParameters::default().set_erodibility(1.0))
            .collect::<_>(),
    );
```

 - **is_outlet** is whether the site can be an outlet (including oceans) or not. If `is_outlet`, the elevation is always set to 0.0.
 - **erodibility** is the erodibility of the site. This is the main parameter that determines the shape of the terrain.

### Generate terrain

```rust
let terrain = terrain_generator.generate().unwrap();
```

Terrains are generated by calling `TerrainGenerator::generate()`.

### Get altitudes

In general you can get the sites and altitudes as vectors by calling `[Terrain]::sites()` and `[Terrain]::altitudes()`.

The indices of each vector correspond to each other.

```rust
let sites = terrain.sites();
let altitudes = terrain.altitudes();
// print
for i in 0..sites.len() {
    println!(
        "Site: ({}, {}), Altitude: {}",
        sites[i].x, sites[i].y, altitudes[i]
    );
}
```

You can also query the altitude of a specific site in the 2D surface model by calling `Terrain2D::get_altitude(Site2D)`.

```rust
let site = Site2D { x, y };
let altitude = terrain.get_altitude(&site);
```

Note that this method uses natural neighbour interpolation (with the [naturalneighbor](https://crates.io/crates/naturalneighbor) package) to interpolate the specific altitude between sites and takes approximately O(logN).

Below is an example of using `get_altitude` to create an image of the generated terrain:

```rust
let img_width = 500;
let img_height = 500;

let mut image_buf = image::RgbImage::new(img_width, img_height);
// calculate the max altitude
let max_altitude = terrain
    .altitudes()
    .iter()
    .fold(std::f64::MIN, |acc, &n| n.max(acc));

for imgx in 0..img_width {
    for imgy in 0..img_height {
        let x = bound_max.x * (imgx as f64 / img_width as f64);
        let y = bound_max.y * (imgy as f64 / img_height as f64);
        let site = Site2D { x, y };
        let altitude = terrain.get_altitude(&site);
        if let Some(altitude) = altitude {
            let color = ((altitude / max_altitude) * 255.0) as u8;
            // put grayscale pixel
            image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb([color, color, color]));
        }
    }
}

image_buf.save("image.png").unwrap();
```

### Reference 

[1] Steer, P.: Short communication: Analytical models for 2D landscape evolution, Earth Surf. Dynam., 9, 1239–1250, https://doi.org/10.5194/esurf-9-1239-2021, 2021.

[2] Guillaume Cordonnier, Jean Braun, Marie-Paule Cani, Bedrich Benes, Eric Galin, et al.. Large Scale Terrain Generation from Tectonic Uplift and Fluvial Erosion. Computer Graphics Forum, 2016, Proc. EUROGRAPHICS 2016, 35 (2), pp.165-175. [⟨10.1111/cgf.12820⟩](https://dx.doi.org/10.1111/cgf.12820). [⟨hal-01262376⟩](https://inria.hal.science/hal-01262376)


## Contributing

Contributions are welcome.
Feel free to open an issue or pull request if you have any problems or suggestions.

## Documentation

The [API reference](https://docs.rs/fastlem/latest/fastlem/) is available for this project.

The [examples](https://github.com/TadaTeruki/fastlem/tree/main/examples) are also useful. Reading the [landscape_evolution.rs](https://github.com/TadaTeruki/fastlem/blob/main/examples/landscape_evolution.rs) is recommended as a first step.

## License

MIT

## Preview of Examples

|**Simple Landscape Evolution**|**Simple Terrain Generation**|
|:---:|:---:|
|![Simple Landscape Evolution](images/out/landscape_evolution.png)|![Simple Terrain Generation](images/out/terrain_generation.png)|
|```$ cargo run --example landscape_evolution --release```|```$ cargo run --example terrain_generation --release```|

|**Advanced Terrain Generation**|**Terrain Generation from Given Parameters**|
|:---:|:---:|
|![Advanced Terrain Generation](images/out/terrain_generation_advanced.png)|![Terrain Generation from Given Parameters](images/out/sample_terrain.png)|
|```$ cargo run --example terrain_generation_advanced --release```|```$ cargo run --example sample_terrain --release```|


