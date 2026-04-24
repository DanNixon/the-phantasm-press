include <./dimensions.scad>;
use <./bottom_panel.scad>;
use <./frontback_panel.scad>;
use <./side_panel.scad>;

// Bottom panel
color("orange") {
  translate([0, 0, -bottom_panel_thickness]) {
    BottomPanel3D();
  }
}

// Side panels
color("green") {
  translate([0, 0, internal[2] / 2]) {
    for(a = [-90, 90]) {
      rotate([0, a, 0]) {
        translate([0, 0, internal[0] / 2]) {
          SidePanel3D();
        }
      }
    }
  }
}

// Front and back panels
color("blue") {
  translate([0, 0, internal[2] / 2]) {
    for(a = [0, 180]) {
      rotate([0, 0, a]) {
        translate([0, -internal[1] / 2, 0]) {
          rotate([90, 0, 0]) {
            FrontBackPanel3D();
          }
        }
      }
    }
  }
}

// Internal area
color("magenta", 0.3) {
  translate([0, 0, internal[2]/2]) {
    cube(internal, center=true);
  }
}
