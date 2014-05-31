use rand::Rng;
use text::Map2DUtil;
use num::Integer;
use world::world::{World, Location};
use world::terrain;

/*
1######B222 Template for herringbone prefabs
1##########
1########## Cells at positions A, B and C must have an open tile.
A########## On each half, the openings A, B and C must be connected.
########### The two halves may or may not be connected.
########### This ensures automatic map connectivity, while not
########### making the map trivially open.
##########B
##########2 The numbered lines are parameters by which the openings
##########2 are positioned. When changing the position of an opening
##########2 for an alternative set, lines with the same symbol must
3*********1 remain at equal length.
3*********1
3*********1
3*********A
3**********
C**********
***********
***********
***********
***********
33333C*****
*/

static CHUNK_W: int = 11;
/*
"\
....... ...
...........
...........
 ..........
...........
...........
...........
..........
...........
...........
...........
...........
...........
...........
..........
...........
 ..........
...........
...........
...........
...........
..... .....",
*/

static CHUNKS: &'static[&'static str] = &[
    /*
"\
%%%%%%%,%%%
%%%%%%%%%%%
%%%%%%%%%%%
,%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%,
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%,
%%%%%%%%%%%
,%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%,%%%%%",
*/

"\
,,,,,,,,,,,
,######|##,
,#..#....#,
,|..+..T.|,
,#..#....#,
,##+######,
,#....#,,x,
,|.T..+,,,,
,#....#,%x,
,###|##,,x,
,,,,,xb,,x,
,,,%,xx,xx%
,,%,,,,,%%%
,,,,,,,,,%%
,,,%,;,,,,,
,,,,;;;,,,,
,%,;;~~,,,,
%%,;~~~~~,,
%,,;~~~~~,%
,,,,~~~,,,%
,,,,,,,,%%%
,,,,,,,%%%%",

"\
%%%%%%%,%%%
%%%%%,,,%%%
%%,,,,,,%%%
,,,,,,,%,,%
%%,,,,,%%,%
%/%,,,,,%%%
%%%%/%%,/%%
%%%%,%%,,,,
%%%%%%%%/%%
%%%%%%%%%%%
%///..//%%%
%/.A..../%%
%.A..A..,%%
%./.a../%,%
%%.A..A/%,,
%%...A,,%%%
,,,,,//%/%%
%%%,,%%%%%%
%%%%,%%/%%%
%%%%%,%%%%%
%%%%%,,%%%%
%%%%%,,%%%%",

"\
%%%%%%%,%%%
%%%%%%,,,%%
%%%%,,,,,%%
,,,,,,,,,%%
%%,,%%,,,%%
%%%%%,,,%%%
%%%%%%,,%,%
%%%%,,,,,,,
%%%%,%%,,,%
%%%%%%%,,%%
%%%%%%%,,%%
%%%%%,,,%%%
%%%%%,%%%%%
%%%%%,%%%%%
%%%%%,,,,,,
%%,,,,,,,,%
,,,,,,,%%%%
%%%,,,%%%%%
%%%%,,,%%%%
%%%%%,,,%%%
%%%%%,,%%%%
%%%%%,,%%%%",

"\
%%%%%%,,%%%
%%%%%%,,,%%
%%%%,,,,,%%
,,,,,,,,,%%
%%,,%%,,,%%
%%%%%,,,%%%
%%%%%%,,%,%
%%%o,,,,,,,
%%%~~%%,,,%
,~%~~~o,,~%
~~=====~~~=
~==========
~====~~~~,%
,,%~~~%%%%%
,%%%%,,,,,,
%%,,,,,,,,%
,,,,,,,%%%%
%%%,,,%%%%%
%%%%,,,%%%%
%%%%%,,,%%%
%%%%%,,%%%%
%%%%%,,%%%%",

"\
#######.###
#######+###
##......###
.+......###
##......###
##g.....###
##......###
##g.....+..
##......###
###+#######
###.#######
###.#######
###.#######
###.#######
###........
#####.#####
......#####
#####.#####
#####.#####
#####.#####
#####.#####
#####.#####",

"\
#######.###
#######.###
#######.###
........###
#######.###
#######.###
#######.###
#######....
###########
###########
###########
###########
###########
###########
#####......
#####.#####
......#####
#####.#####
#####.#####
#####.#####
#####.#####
#####.#####",

"\
#######.###
#######+###
##.......##
.+...#...##
##..###..##
##...#...##
##.#...#.##
##.......+.
##.......##
##.#...#.##
##.......##
##.......##
##.#...#.##
##.......##
##.......+.
##.#...#.##
.+...#...##
##..###..##
##...#...##
##.......##
#####+#####
#####.#####",

"\
*******.***
****#...***
#####...#**
.......##**
##.....#..*
*..##..#..*
**..#++##**
**#|#......
**....!..**
**..!.....*
**..XX..!.*
**!.XXX..**
**...XX..**
**X......**
**XXXXX....
**.XXX...**
.......X.**
**..!..XX**
***....XX**
****..**XX*
*****.*XXX*
*****.*****",

"\
#######.###
#######.###
#######.###
........###
#.......###
#.......###
#.......###
#..........
###=====###
###=====###
###=====###
###=====###
###IIIII###
###.....###
###........
###.....###
........###
###.....###
###.....###
###.....###
#####.#####
#####.#####",
];

pub trait MapGen {
    fn gen_herringbone<R: Rng>(&mut self, rng: &mut R);
}

impl MapGen for World {
    // http://nothings.org/gamedev/herringbone/
    fn gen_herringbone<R: Rng>(&mut self, rng: &mut R) {
        for cy in range(-3, 4) {
            for cx in range(-3, 4) {
                let chunk = rng.choose(CHUNKS).unwrap();
                for (glyph, x, y) in chunk.chars().map2d() {
                    let terrain = match glyph {
                        '.' => terrain::Floor,
                        '#' => terrain::Wall,
                        '~' => terrain::Shallows,
                        '=' => terrain::Water,
                        ',' => terrain::Grass,
                        '+' => terrain::Door,
                        '*' => terrain::Rock,
                        'X' => terrain::Magma,
                        '|' => terrain::Window,
                        '%' => terrain::Tree,
                        '/' => terrain::DeadTree,
                        'x' => terrain::Fence,
                        'o' => terrain::Stone,
                        'A' => terrain::Menhir,
                        'g' => terrain::Grave,
                        'b' => terrain::Barrel,
                        'T' => terrain::Table,
                        'a' => terrain::Altar,
                        'I' => terrain::Bars,
                        '!' => terrain::Stalagmite,
                        ';' => terrain::TallGrass,
                        _ => terrain::Void
                    };
                    let (ax, ay) = herringbone_map((cx, cy), (x, y));
                    self.terrain_set(Location::new(ax as i8, ay as i8), terrain);
                }
            }
        }
    }
}

// Map in-chunk coordinates to on-map coordinates based on chunk position in
// the herringbone chunk grid.
fn herringbone_map(chunk_pos: (int, int), in_chunk_pos: (int, int)) -> (int, int) {
    let (cx, cy) = chunk_pos;
    let (div, m) = cx.div_mod_floor(&2);
    let (x, y) = in_chunk_pos;

    let origin_x = div * CHUNK_W + cy * CHUNK_W;
    let origin_y = cy * CHUNK_W - m * CHUNK_W - 3 * div * CHUNK_W;

    if m == 0 {
        (origin_x + x, origin_y + y)
    } else {
        (origin_x + y, origin_y + x)
    }
}