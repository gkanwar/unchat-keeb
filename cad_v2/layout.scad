use <cherry.scad>;
use <adafruit_qt_py.scad>;
use <rj45.scad>;

cdx = 19;
cdy = 19;

$fn = 32;

module cherry_1u_array(nx, ny, letters, with_kc) {
  for(i = [0:nx-1]) {
    for(j = [0:ny-1]) {
      translate([i*cdx, j*cdy, 0])
      cherry_1u(s=letters[i + nx*(ny-j-1)], with_kc=with_kc);
    }
  }
}

module switches(with_kc=true) {
  cherry_1u_array(4, 3,
    ["W", "E", "R", "T",
    "S", "D", "F", "G",
    "X", "C", "V", "B"],
    with_kc=with_kc);
  translate([-2*cdx, 0*cdy, 0])
  cherry_1u_array(2, 3,
    [" ", "Q", " ", "A", " ", "Z"],
    with_kc=with_kc);
  translate([1.5*cdx, -1.1*cdy, 0])
  cherry_1u(with_kc=with_kc);
  translate([2.7*cdx, -1.25*cdy, 0])
  rotate(-15)
  cherry_1u(with_kc=with_kc);
}

module screwholes() {
  sd=2.2;
  translate([-13, -13, 0])
  circle(d=sd);
  translate([-68, -13, 0])
  circle(d=sd);
  translate([-37, -31, 0])
  circle(d=sd);
  translate([-5.5*cdx, 2*cdy, 0])
  circle(d=sd);
  translate([-1.5*cdx, 2*cdy, 0])
  circle(d=sd);
  translate([-5.5*cdx, 0*cdy, 0])
  circle(d=sd);
}

module outline(plate_only=false) {
  center_t2 = [(2.8+.5)*cdx, (-1.25+.5)*cdy];
  t2_rot = -15;
  cornerR_t2 = [
    center_t2[0] + 0.5*cdx*cos(t2_rot) + 0.5*cdy*sin(t2_rot),
    center_t2[1] + 0.5*cdx*sin(t2_rot) - 0.5*cdy*cos(t2_rot)
  ];
  translate([-0.5*cdx, -0.5*cdy, 0])
  union() {
    // 3x5 main area
    polygon([
      [-1*cdx,0], [4*cdx,0],
      [4*cdx,3*cdy], [-1*cdx,3*cdy]
    ]);
    if (!plate_only) {
      // electronics area
      polygon([
        [4*cdx,0], [5.25*cdx,0],
        [5.25*cdx,3*cdy], [4*cdx,3*cdy]
      ]);
      // full thumb cluster
      polygon([
        [-1*cdx,1.5*cdy], [-1*cdx,0],
        [0.5*cdx,0], [1.5*cdx,-1.1*cdy],
        cornerR_t2,
        [4.75*cdx,0*cdy], [4.75*cdx,1.5*cdy]
      ]);
    }
    else {
      // reduced thumb cluster
      polygon([
        [-1*cdx,1.5*cdy], [-1*cdx,0],
        [0.5*cdx,0], [1.5*cdx,-1.1*cdy],
        cornerR_t2,
        [4*cdx,0*cdy], [4*cdx,1.5*cdy]
      ]);
    }
    // 3x1 outermost column
    polygon([
      [-2*cdx,0*cdy],[-1*cdx,0*cdy],
      [-1*cdx,3*cdy],[-2*cdx,3*cdy]
    ]);
  }
}

hand_margin = 4*cdx;

module pcb_footprint() {
  difference() {
    translate([-hand_margin, 0, 0])
    offset(r = -2) offset(r = 4)
    outline(plate_only=false);
    union() {
      translate([0.75*cdx+2, .15*cdy, 0])
      rotate([0, 0, 90])
      rj45_footprint();
      screwholes();
    }
  }
}

module plate_footprint() {
  translate([-hand_margin, 0, 0])
  offset(r = -2) offset (r = 4)
  outline(plate_only=true);
}

module bottom_plate_outline() {
  difference() {
    plate_footprint();
    screwholes();
  }
}

module bottom_plate() {
  color("white", 0.2)
  translate([0, 0, -3])
  linear_extrude(3.0, center=false)
  bottom_plate_outline();
}

module top_plate_outline() {
  difference() {
    plate_footprint();
    union() {
      projection(cut=true)
      do_switches(with_kc=false);
      screwholes();
    }
  }
}

module top_plate() {
  color("gray")
  translate([0, 0, -1.5])
  linear_extrude(1.5, center=false)
  top_plate_outline();
}

module pcb() {
  color("darkgrey")
  translate([0, 0, -1.6])
  linear_extrude(1.6, center=false)
  pcb_footprint();
}

module do_switches(with_kc=true) {
  translate([-hand_margin, 0, 0])
  switches(with_kc=with_kc);
}

module components() {
  translate([0.15*cdy, 2*cdy, 0])
  qtpy();
  translate([.75*cdx+2, .15*cdy, 0])
  rotate([0, 0, 90])
  rj45();
}

do_switches(with_kc=false);
pcb();
components();
translate([0, 0, -3])
bottom_plate();
translate([0, 0, 3])
top_plate();