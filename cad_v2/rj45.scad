h = 11.45;

module rj45() {
  color("Silver")
  translate([0, 0, h-8.65+0.25+0.35])
  rotate([90, 0, 0])
  import("../components/rj45-2041126-1.stl");
  /* w = 15.30;
  d = 17.74;
  h = 12.70;
  ph = 16.20 - h;
  sx = (w-11.43)/2;
  translate([-w/2, -d/2, 0])
  color("silver") union() {
    translate([sx, 7.62, -ph/2])
    cylinder(ph, 1.60, 1.60, center=true);
    translate([w-sx, 7.62, -ph/2])
    cylinder(ph, 1.60, 1.60, center=true);
    difference() {
      cube([w, d, h]);
      translate([0.05*w, 0, 0.15*h])
      cube([0.9*w, 0.9*d, 0.6*h]);
    }
  } */
}
module rj45_footprint() {
  translate([-7.0, 0, 0])
  square([14.0, 10.9]);
}

rj45();