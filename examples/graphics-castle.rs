//! Title: "The Computer Graphics Castle"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;
use std::convert::TryInto;
use std::collections::{VecDeque, HashSet};

use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Cylinder, MeshData, Shading},
    kdtree::KDMesh,
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};

fn main() -> Result<(), Box<dyn Error>> {
    let scene = HierScene {
        root: SceneNode::from(vec![
            castle()?
                .scaled(1.4)
                .translated((0.0, 0.0, -229.0))
                .into(),

            lake()?.into(),
            land()?.into(),
            outdoor_maze().into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 65.0, y: 130.0, z: -120.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },

            // Dim lights inside each tower to light inside
            Light {
                position: Vec3 {x: -42.0, y: 112.0, z: -229.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 42.0, y: 112.0, z: -229.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 0.0, y: 129.0, z: -260.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 123.722778, 587.455566).into(),
        center: (0.0, -28.862762, -542.286621).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    // let mut image = Image::new("graphics-castle.png", 1920, 1080)?;
    let mut image = Image::new("graphics-castle.png", 533, 300)?;

    // image.slice_mut((152, 128), (382, 162)).render::<RenderProgress, _>(&scene, cam,
    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save()?)
}

fn castle() -> Result<SceneNode, Box<dyn Error>> {
    let mat_castle_walls = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_castle_door = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_castle_window_frames = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_stairs_side = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_puppet = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let castle_model = Arc::new(MeshData::load_obj("assets/castle.obj")?);
    let castle_window_frames_model = Arc::new(MeshData::load_obj("assets/castle_window_frames.obj")?);
    let castle_door_model = Arc::new(MeshData::load_obj("assets/castle_door.obj")?);
    let castle_door_arch_model = Arc::new(MeshData::load_obj("assets/castle_door_arch.obj")?);
    let castle_stairs_side = Arc::new(MeshData::load_obj("assets/castle_stairs_side.obj")?);
    let puppet_castle_left_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_left_tower.obj")?);
    let puppet_castle_right_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_right_tower.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(KDMesh::new(&castle_model, Shading::Flat), mat_castle_walls.clone()))
            .translated((0.0, 30.0, -30.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_window_frames_model, Shading::Flat), mat_castle_window_frames.clone()))
            .translated((0.0, 83.5746, -2.25))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&castle_door_model, Shading::Flat), mat_castle_door.clone()))
            .translated((0.0, 21.739681, 10.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_door_arch_model, Shading::Flat), mat_castle_door.clone()))
            .translated((0.0, 42.0, 9.0))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((-11.0, 5.0, 19.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((11.0, 5.0, 19.0))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&puppet_castle_left_tower_model, Shading::Smooth), mat_puppet.clone()))
            .translated((30.0, 33.6, 19.0))
            .into(),
        SceneNode::from(Geometry::new(Cylinder, mat_castle_walls.clone()))
            .scaled(10.0)
            .translated((30.0, 5.0, 20.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&puppet_castle_right_tower_model, Shading::Smooth), mat_puppet.clone()))
            .translated((-30.0, 33.6, 19.0))
            .into(),
        SceneNode::from(Geometry::new(Cylinder, mat_castle_walls.clone()))
            .scaled(10.0)
            .translated((-30.0, 5.0, 20.0))
            .into(),
    ]))
}

fn lake() -> Result<SceneNode, Box<dyn Error>> {
    let mat_water = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.1},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.9,
        glossy_side_length: 1.0,
        refraction_index: WATER_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_dirt = Arc::new(Material {
        // Color of algae makes the water blue!
        diffuse: Rgb {r: 0.592, g: 0.671, b: 0.055},
        ..Material::default()
    });

    let castle_water_dirt_model = Arc::new(MeshData::load_obj("assets/castle_water_dirt.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(KDMesh::new(&castle_water_dirt_model, Shading::Flat), mat_dirt))
            .translated((0.0, -62.0, 125.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, mat_water))
            .scaled((640.0, 125.0, 250.0))
            .translated((0.0, -62.0, 125.0))
            .into(),
    ]))
}

fn land() -> Result<SceneNode, Box<dyn Error>> {
    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.376, g: 0.502, b: 0.22},
        ..Material::default()
    });

    let castle_hill_model = Arc::new(MeshData::load_obj("assets/castle_hill.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(KDMesh::new(&castle_hill_model, Shading::Smooth), mat_grass.clone()))
            .translated((0.0, 3.75, -15.75))
            .scaled(1.4)
            .translated((0.0, 0.0, -229.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, mat_grass.clone()))
            .scaled((2560.0, 132.0, 1040.0))
            .translated((0.0, -65.0, -520.0))
            .into(),
    ]))
}

fn outdoor_maze() -> SceneNode {
    // Needs to be a size that works proportionally with the rest of the scene
    let cell_width = 8.0;
    let cell_length = cell_width;

    // Chosen to be evenly divisible by cell_width
    let maze_width = 1664.0;
    // Chosen to be evenly divisible by cell_length
    let maze_length = 1280.0;
    // Constant for all cells / the whole maze
    let maze_height = 8.0;

    // Area around the castle
    // Chosen to be evenly divisible by cell_width
    let castle_area_width = 312.0;
    // Chosen to be evenly divisible by cell_length
    let castle_area_length = 160.0;
    // Centered at the castle but then offset relative to maze pos (see last line of this function)
    let castle_pos = Vec3 {x: 0.0, y: 0.0, z: -660.0 - (-260.0)};

    // Entrance position (assumed to be in the bottom row)
    let entrance_x = -100.0;

    let maze_cols = (maze_width / cell_width) as usize;
    let maze_rows = (maze_length / cell_length) as usize;

    // Assume last row
    let entrance_row = maze_rows - 1;
    let entrance_col = ((entrance_x + maze_width / 2.0) / cell_width) as usize;

    // Find the boundary around the castle
    let back_corner_row = ((castle_pos.z - castle_area_length/2.0 + maze_length / 2.0) / cell_length) as usize;
    let back_corner_col = ((castle_pos.x - castle_area_width/2.0 + maze_width / 2.0) / cell_width) as usize;
    let front_corner_row = ((castle_pos.z + castle_area_length/2.0 + maze_length / 2.0) / cell_length) as usize;
    let front_corner_col = ((castle_pos.x + castle_area_width/2.0 + maze_width / 2.0) / cell_width) as usize;

    let mut maze = Maze::new(maze_rows, maze_cols);
    maze.reserve((back_corner_row, back_corner_col), (front_corner_row, front_corner_col));
    // maze.fill_maze((entrance_row, entrance_col));

    {
        let mut m = Maze::new(10, 10);
        m.reserve((1, 1), (3, 3));
        m.fill_maze((9, 2));

        for r in &m.cells {
            for c in r {
                match c {
                    Cell::Empty => print!("_"),
                    Cell::Wall => print!("W"),
                }
            }
            println!();
        }
    }

    let mat_maze = Arc::new(Material {
        diffuse: Rgb {r: 0.038907, g: 0.117096, b: 0.040216},
        ..Material::default()
    });

    let mut nodes = Vec::new();
    for (i, row) in maze.cells.iter().enumerate() {
        let z = i as f64 * cell_length - maze_length / 2.0;

        for (j, cell) in row.iter().enumerate() {
            match cell {
                Cell::Empty => continue,
                Cell::Wall => {},
            }

            let x = j as f64 * cell_width - maze_width / 2.0;

            nodes.push(
                SceneNode::from(Geometry::new(Cube, mat_maze.clone()))
                    .scaled((cell_width, maze_height, cell_length))
                    .translated((x, 0.0, z))
                    .into(),
            );
        }
    }

    // Translate the maze to its correct position in the scene
    SceneNode::from(nodes).translated((0.0, maze_height/2.0 + 1.0, -660.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
}

#[derive(Debug, Clone)]
struct Maze {
    /// The rows of the maze, stored row-wise
    cells: Vec<Vec<Cell>>,
}

impl Maze {
    pub fn new(rows: usize, cols: usize) -> Self {
        // Rest of the code relies on these being non-empty
        assert!(rows > 0 && cols > 0);

        Self {
            cells: vec![vec![Cell::Wall; cols]; rows],
        }
    }

    /// Reserves the given range of cells so that no walls will be placed there.
    ///
    /// The ranges are inclusive on both ends.
    pub fn reserve(&mut self, (row1, col1): (usize, usize), (row2, col2): (usize, usize)) {
        for row in row1..=row2 {
            for col in col1..=col2 {
                self.cells[row][col] = Cell::Empty;
            }
        }
    }

    /// Generate the maze by filling the cells starting at the given point
    pub fn fill_maze(&mut self, (start_row, start_col): (usize, usize)) {
        let rows = self.cells.len();
        let cols = self.cells[0].len();

        // Utility function for finding the adjacents of a given cell and storing the result in a
        // pre-allocated array
        let find_adjacents = |adjacents: &mut [_; 4], row, col| {
            adjacents[0] = if row > 0 { Some((row - 1, col)) } else { None };
            adjacents[1] = if row < rows-1 { Some((row + 1, col)) } else { None };
            adjacents[2] = if col > 0 { Some((row, col - 1)) } else { None };
            adjacents[3] = if col < cols-1 { Some((row, col + 1)) } else { None };
        };

        // Utility function for finding the pairs of adjacents of a given cell and storing the
        // result in a pre-allocated array
        let find_adjacent_pairs = |adjacent_pairs: &mut [_; 2], row, col| {
            let above = if row > 0 { Some((row - 1, col)) } else { None };
            let below = if row < rows-1 { Some((row + 1, col)) } else { None };
            let left = if col > 0 { Some((row, col - 1)) } else { None };
            let right = if col < cols-1 { Some((row, col + 1)) } else { None };

            adjacent_pairs[0] = above.and_then(|a| below.map(|b| (a, b)));
            adjacent_pairs[1] = left.and_then(|a| right.map(|b| (a, b)));
        };

        // Want a random maze but want the same one every time
        let mut rng = StdRng::seed_from_u64(193920103958);

        // Reuse memory to store adjacents
        let mut adjacents = [None; 4];
        let mut adjacent_pairs = [None; 2];

        // Using a randomized Prim's algorithm as described here:
        // https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_Prim's_algorithm
        self.cells[start_row][start_col] = Cell::Empty;

        let mut walls = VecDeque::new();
        find_adjacents(&mut adjacents, start_row, start_col);
        walls.extend(adjacents.iter().flatten().cloned());

        let mut seen = HashSet::new();
        while let Some((row, col)) = walls.pop_front() {
            if seen.contains(&(row, col)) {
                continue;
            }
            seen.insert((row, col));

            // Go through adjacent pairs in a random order so the maze has some variation
            find_adjacent_pairs(&mut adjacent_pairs, row, col);
            adjacent_pairs.shuffle(&mut rng);
            for adj_pair in &adjacent_pairs {
                if let &Some(((adj1_row, adj1_col), (adj2_row, adj2_col))) = adj_pair {
                    // If only one of the cells divided by this wall is empty, make the wall into a
                    // passage that connects to that empty tile
                    match (self.cells[adj1_row][adj1_col], self.cells[adj2_row][adj2_col]) {
                        (Cell::Empty, Cell::Wall) => {
                            // Make a passage
                            self.cells[row][col] = Cell::Empty;

                            // Add the wall to the walls list
                            walls.push_back((adj2_row, adj2_col));

                            // Stop processing adjacents once we've created a passage
                            break;
                        },
                        (Cell::Wall, Cell::Empty) => {
                            // Same as above but for the other adjacent
                            self.cells[row][col] = Cell::Empty;

                            // Add the wall to the walls list
                            walls.push_back((adj1_row, adj1_col));

                            // Stop processing adjacents once we've created a passage
                            break;
                        },
                        // Cannot create a passage
                        _ => {}
                    }
                }
            }

            // Shuffle the list of walls for next time so we pick a random wall every time
            let (front, back) = walls.as_mut_slices();
            front.shuffle(&mut rng);
            back.shuffle(&mut rng);

            for r in &self.cells {
                for c in r {
                    match c {
                        Cell::Empty => print!("_"),
                        Cell::Wall => print!("W"),
                    }
                }
                println!();
            }
            println!();
        }
    }
}
