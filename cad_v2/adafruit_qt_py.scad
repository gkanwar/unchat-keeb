module qtpy() {
  inch_to_mm = 25.4;
  translate([-inch_to_mm*0.70/2, -inch_to_mm*0.81/2, 0])
  color("FireBrick")
  import("../components/4600 QTPy.step.stl");
}

qtpy();