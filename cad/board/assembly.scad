include <dimensions.scad>;
use <bottom_plate.scad>;
use <card_reader_spacer.scad>;
use <control_board_spacer.scad>;
use <led_ring_mount.scad>;
use <pillar.scad>;
use <side_panel.scad>;
use <side_panel_cable_exit.scad>;
use <top_plate_lower.scad>;
use <top_plate_upper.scad>;

// Bottom plate
color("orange") {
    translate([0, 0, -bottom_plate_thickness]) {
        BottomPlate3D();
    }
}

// Side panels
color("wheat") {
    for(a = [0 : 45 : 359]) {
        rotate(a) {
            translate([0, (side_panel_thickness / 2) + side_panel_offset, 0]) {
                rotate([90, 0, 0]) {
                    if(a == 0) {
                        SidePanelCableExit3D();
                    } else {
                        SidePanel3D();
                    }
                }
            }
        }
    }
}

// Pillars
color("purple") {
    for(a = [45 / 2 : 45 : 359]) {
        rotate(a) {
            translate([pillar_offset, 0, 0]) {
                Pillar3D();
            }
        }
    }
}

// Control board spacer
color("cyan") {
    translate([0, control_board_offset, 0]) {
        ControlBoardSpacer3D();
    }
}

// LED ring mounts
color("cyan") {
    for(a = [45 : 90 : 359]) {
        rotate(a) {
            translate([0, led_ring_mount_offset, 0.1]) {
                LedRingMount3D();
            }
        }
    }
}

// Card reader spacers
color("cyan") {
    for(a = [0 : 90 : 359]) {
        rotate(a) {
            translate([0, card_reader_offset, internal_height - card_reader_spacer_thickness - 0.01]) {
                CardReaderSpacer3D();
            }
        }
    }
}

// Top plate - lower
color("grey", 0.5) {
    translate([0, 0, internal_height]) {
        TopPlateLower3D();
    }
}

// Top plate - upper
color("lightgrey", 0.5) {
    translate([0, 0, internal_height + top_plate_thickness + 0.01]) {
        TopPlateUpper3D();
    }
}
