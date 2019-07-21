//! Title: "The Computer Graphics Castle"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;
use std::collections::{VecDeque, HashSet};

use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Cylinder, MeshData, Shading},
    kdtree::KDMesh,
    material::{Material, WATER_REFRACTION_INDEX, WINDOW_GLASS_REFRACTION_INDEX},
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
                position: Vec3 {x: -30.0, y: 80.0, z: 0.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 30.0, y: 80.0, z: 0.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 0.0, y: 93.0, z: -23.0},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 115.785934, 528.691223).into(),
        center: (0.0, -130.492584, -1294.752441).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = Image::new("graphics-castle.png", 1920, 1080)?;

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

    let mat_ceiling_glass = Arc::new(Material {
        diffuse: Rgb {r: 0.088418, g: 0.249559, b: 0.067798},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 100.0,
        reflectivity: 0.8,
        refraction_index: WINDOW_GLASS_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_window_glass = Arc::new(Material {
        diffuse: Rgb {r: 0.091208, g: 0.14981, b: 0.084696},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 100.0,
        reflectivity: 1.0,
        refraction_index: WINDOW_GLASS_REFRACTION_INDEX,
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

    let mat_teapot = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.214141, b: 0.794883},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let castle_model = Arc::new(MeshData::load_obj("assets/castle.obj")?);
    let castle_window_frames_model = Arc::new(MeshData::load_obj("assets/castle_window_frames.obj")?);
    let castle_glass_ceilings_model = Arc::new(MeshData::load_obj("assets/castle_glass_ceilings.obj")?);
    let castle_door_model = Arc::new(MeshData::load_obj("assets/castle_door.obj")?);
    let castle_door_arch_model = Arc::new(MeshData::load_obj("assets/castle_door_arch.obj")?);
    let castle_stairs_side = Arc::new(MeshData::load_obj("assets/castle_stairs_side.obj")?);
    let puppet_castle_left_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_left_tower.obj")?);
    let puppet_castle_right_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_right_tower.obj")?);
    let teapot_model = Arc::new(MeshData::load_obj("assets/teapot.obj")?);

    Ok(SceneNode::from(vec![
        // Main castle body
        SceneNode::from(Geometry::new(KDMesh::new(&castle_model, Shading::Flat), mat_castle_walls.clone()))
            .translated((0.0, 30.0, -30.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_window_frames_model, Shading::Flat), mat_castle_window_frames.clone()))
            .translated((0.0, 83.5746, -2.25))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_glass_ceilings_model, Shading::Flat), mat_ceiling_glass.clone()))
            .translated((0.0, 96.0, -23.0))
            .into(),

        // Windows
        SceneNode::from(Geometry::new(Cube, mat_window_glass.clone()))
            .scaled((9.1, 1.0, 12.7))
            .rotated_x(Radians::from_degrees(90.0))
            .translated((-30.0, 70.7, 12.7))
            .into(),
        SceneNode::from(Geometry::new(Cube, mat_window_glass.clone()))
            .scaled((9.1, 1.0, 12.7))
            .rotated_x(Radians::from_degrees(90.0))
            .translated((30.0, 70.7, 12.7))
            .into(),
        SceneNode::from(Geometry::new(Cube, mat_window_glass.clone()))
            .scaled((13.4, 1.0, 18.8))
            .rotated_x(Radians::from_degrees(90.0))
            .translated((0.0, 79.4, -2.9))
            .into(),

        // Tower objects
        SceneNode::from(Geometry::new(KDMesh::new(&teapot_model, Shading::Smooth), mat_teapot.clone()))
            .scaled(0.64)
            .rotated_y(Radians::from_degrees(-60.0))
            .translated((0.0, 74.0, -23.0))
            .into(),

        // Door
        SceneNode::from(Geometry::new(KDMesh::new(&castle_door_model, Shading::Flat), mat_castle_door.clone()))
            .translated((0.0, 21.739681, 10.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_door_arch_model, Shading::Flat), mat_castle_door.clone()))
            .translated((0.0, 42.0, 9.0))
            .into(),

        // Stairs
        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((-11.0, 5.0, 19.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((11.0, 5.0, 19.0))
            .into(),

        // Statues / Guardians
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
    let cell_width = 4.0;
    let cell_length = cell_width;

    // Chosen to be evenly divisible by cell_width
    let maze_width = 1328.0;
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
    maze.fill_maze((entrance_row, entrance_col));

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
            // Leave the first and last row/column untouched
            adjacents[0] = if row > 1 { Some((row - 1, col)) } else { None };
            adjacents[1] = if row < rows-2 { Some((row + 1, col)) } else { None };
            adjacents[2] = if col > 1 { Some((row, col - 1)) } else { None };
            adjacents[3] = if col < cols-2 { Some((row, col + 1)) } else { None };
        };

        // Utility function for finding the diagonal adjacents of a given cell and storing the
        // result in a pre-allocated array
        let find_diagonal_adjacents = |adjacents: &mut [_; 4], row, col| {
            // Leave the first and last row/column untouched
            adjacents[0] = if row > 1 && col > 1 { Some((row - 1, col - 1)) } else { None };
            adjacents[1] = if row < rows-2 && col > 1 { Some((row + 1, col - 1)) } else { None };
            adjacents[2] = if row > 1 && col < cols-2 { Some((row - 1, col + 1)) } else { None };
            adjacents[3] = if row < rows-2 && col < cols-2 { Some((row + 1, col + 1)) } else { None };
        };

        // Want a random maze but want the same one every time
        let mut rng = StdRng::seed_from_u64(193920103958);

        // Reuse memory to store adjacents
        let mut adjacents = [None; 4];

        let mut walls = VecDeque::new();
        let mut seen = HashSet::new();

        // Set the start cell to empty and explore its adjacents
        self.cells[start_row][start_col] = Cell::Empty;
        find_adjacents(&mut adjacents, start_row, start_col);
        walls.extend(adjacents.iter().flatten().cloned());

        while let Some((row, col)) = walls.pop_front() {
            if seen.contains(&(row, col)) {
                continue;
            }
            seen.insert((row, col));

            if self.cells[row][col] == Cell::Empty {
                // Cell is probably reserved
                continue;
            }

            // Diagonal lines of empty cells look ugly, so we filter them out
            find_diagonal_adjacents(&mut adjacents, row, col);
            let empty_diagonals = adjacents.iter()
                .flatten()
                .filter(|&&(row, col)| self.cells[row][col] == Cell::Empty)
                .count();
            if empty_diagonals > 1 {
                continue;
            }

            // Compute adjacents later so we can reuse them
            find_adjacents(&mut adjacents, row, col);
            let empty_adjs = adjacents.iter()
                .flatten()
                .filter(|&&(row, col)| self.cells[row][col] == Cell::Empty)
                .count();

            // Don't want to inadvertantly create any loops
            if empty_adjs > 1 {
                continue;
            }

            // Add the cell to the maze
            self.cells[row][col] = Cell::Empty;

            // Add its adjacent walls to the queue in a random order
            adjacents.shuffle(&mut rng);
            let mut adj_walls = adjacents.iter()
                .flatten()
                .cloned()
                .filter(|&(row, col)| self.cells[row][col] == Cell::Wall);

            // Go depth first to create longer paths
            if let Some(wall) = adj_walls.next() {
                walls.push_front(wall);
            }
            walls.extend(adj_walls);
        }
    }
}
