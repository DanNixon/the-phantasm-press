include <dimensions.scad>;

module Spacer3D() {
  rotate([0, 90, 0]) {
    difference() {
      // Main body
      cylinder(h = internal_width, d = 10, center = true, $fn = 12);

      // Brass insert holes
      for(a = [0, 180]) {
        rotate([0, a, 0]) {
          translate([0, 0, -(internal_width / 2) - 0.01]) {
            cylinder(h = 40, d = 4.5, $fn = 8);
          }
        }
      }
    }
  }
}

Spacer3D();
