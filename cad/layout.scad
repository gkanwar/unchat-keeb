use <choc_v1.scad>;
use <pico.scad>;

cdx = 18;
cdy = 17;

module choc_1u_array(nx, ny, letters, with_kc) {
  for(i = [0:nx-1]) {
    for(j = [0:ny-1]) {
      translate([i*cdx, j*cdy, 0])
      choc_1u(s=letters[i + nx*(ny-j-1)], with_kc=with_kc);
    }
  }
}

module layout_lh(with_kc=true) {
  choc_1u_array(4, 3,
    ["W", "E", "R", "T",
    "S", "D", "F", "G",
    "X", "C", "V", "B"],
    with_kc=with_kc);
  translate([-2*cdx, -0.5*cdy, 0])
  choc_1u_array(2, 3,
    [" ", "Q", " ", "A", " ", "Z"],
    with_kc=with_kc);
  translate([1.25*cdx, -1.1*cdy, 0])
  choc_1u(with_kc=with_kc);
  translate([2.3*cdx, -1.5*cdy, 0])
  rotate(-15)
  choc_1p5uR(with_kc=with_kc);
  translate([3.3*cdx, -1.5*cdy, 0])
  rotate(-15)
  choc_1p5uR(with_kc=with_kc);
}

module layout_rh(with_kc=true) {
  choc_1u_array(4, 3,
    ["Y", "U", "I", "O",
    "H", "J", "K", "L",
    "N", "M", ",", "."],
    with_kc=with_kc);
  translate([4*cdx, -0.5*cdy, 0])
  choc_1u_array(2, 3,
    ["P", " ", ";", " ", "/", " "],
    with_kc=with_kc);
  translate([(3-1.25)*cdx, -1.1*cdy, 0])
  choc_1u(with_kc=with_kc);
  translate([(3-2.3)*cdx, -1.5*cdy, 0])
  rotate(15)
  choc_1p5uR(with_kc=with_kc);
  translate([(3-3.3)*cdx, -1.5*cdy, 0])
  rotate(15)
  choc_1p5uR(with_kc=with_kc);
}

module pcb_half() {
  center_t1 = [(2.3+.5)*cdx, (-1.5+.5)*cdy];
  center_t2 = [(3.3+.5)*cdx, (-1.5+.5)*cdy];
  t1_rot = -15;
  t2_rot = -15;
  cornerL_t1 = [
    center_t1[0] - 0.5*cdx*cos(t1_rot) + 0.75*cdy*sin(t1_rot),
    center_t1[1] - 0.5*cdx*sin(t1_rot) - 0.75*cdy*cos(t1_rot)
  ];
  cornerR_t1 = [
    center_t1[0] + 0.5*cdx*cos(t1_rot) + 0.75*cdy*sin(t1_rot),
    center_t1[1] + 0.5*cdx*sin(t1_rot) - 0.75*cdy*cos(t1_rot)
  ];
  cornerR_t2 = [
    center_t2[0] + 0.5*cdx*cos(t2_rot) + 0.75*cdy*sin(t2_rot),
    center_t2[1] + 0.5*cdx*sin(t2_rot) - 0.75*cdy*cos(t2_rot)
  ];
  cornerU_t2 = [
    center_t2[0] + 0.5*cdx*cos(t2_rot) - 0.75*cdy*sin(t2_rot),
    center_t2[1] + 0.5*cdx*sin(t2_rot) + 0.75*cdy*cos(t2_rot)
  ];
  translate([-0.5*cdx, -0.5*cdy, 0])
  union() {
    polygon([
      [0,0], [4*cdx,0],
      [4*cdx,3*cdy], [0,3*cdy]
    ]);
    polygon([
      [0,0], [0,-0.5*cdy],
      cornerL_t1, cornerR_t1,
      cornerR_t2, cornerU_t2,
      [4*cdx,0]
    ]);
    polygon([
      [-2*cdx,-0.5*cdy],[0,-0.5*cdy],
      [0,2.5*cdy],[-2*cdx,2.5*cdy]
    ]);
  }
}

module pcb_mid() {
  polygon([
    [-2.5*cdx,-2.2*cdy], [2.5*cdx,-2.2*cdy],
    [2.5*cdx,2*cdy], [-2.5*cdx,2*cdy]
  ]);
}

hand_margin = 2*cdx;

module pcb() {
  $fs = 0.05;
  color("IndianRed")
  translate([0, 0, -1.6])
  linear_extrude(1.6, center=false)
  offset(r = 1)
  union() {
    translate([-hand_margin, 0, 0])
    rotate(-15)
    translate([-3*cdx, 0, 0])
    pcb_half();
    translate([hand_margin, 0, 0])
    rotate(15)
    mirror([1,0,0])
    translate([-3*cdx, 0, 0])
    pcb_half();
    pcb_mid();
  }
}

module layout(with_kc=true, with_pico=true) {
  translate([-hand_margin, 0, 0])
  rotate(-15)
  translate([-3*cdx, 0, 0])
  layout_lh(with_kc=with_kc);
  translate([hand_margin, 0, 0])
  rotate(15)
  layout_rh(with_kc=with_kc);
  if (with_pico) {
    translate([0, 0, 0])
    pico();
  }
}

layout();
pcb();