include <./dimensions.scad>;
use <./bottom_panel.scad>;
use <./front_panel.scad>;
use <./side_panel.scad>;
use <./spacer.scad>;
use <./top_panel.scad>;

// Side panels
color("orange") {
  dx = (internal_width + side_panel_thickness) / 2;
  for(x = [-dx - 2, dx]) {
    translate([x, 0, 0]) {
      SidePanel3D();
    }
  }
}

// Spacers
color("red") {
  for(p = spacer_positions) {
    translate([0, p[0], p[1]]) {
      Spacer3D();
    }
  }
}

// Bottom panel
color("cyan") {
  BottomPanel3D();
}

// Front panel
color("green") {
  translate([0, -internal_depth / 2, (internal_front_height - bottom_panel_thickness) / 2]) {
    FrontPanel3D();
  }
}

// Top panel
color("blue") {
  translate([0, 0, top_panel_assembly_height()]) {
    rotate([top_panel_angle(), 0, 0]) {
      TopPanel3D();
    }
  }
}

// Internal volume
color("magenta", 0.4) {
  polyhedron(
    points = [
      [-internal_width / 2, -internal_depth / 2, 0],
      [ internal_width / 2, -internal_depth / 2, 0],
      [ internal_width / 2,  internal_depth / 2, 0],
      [-internal_width / 2,  internal_depth / 2, 0],
      [-internal_width / 2, -internal_depth / 2, internal_front_height],
      [ internal_width / 2, -internal_depth / 2, internal_front_height],
      [ internal_width / 2,  internal_depth / 2, internal_rear_height],
      [-internal_width / 2,  internal_depth / 2, internal_rear_height],
    ],
    faces = [
      [0, 1, 2, 3],
      [4, 5, 1, 0],
      [7, 6, 5, 4],
      [5, 6, 2, 1],
      [6, 7, 3, 2],
      [7, 4, 0, 3],
    ]
  );
}
