// The ray tracer code in this file is written by Adam Burmister. It
// is available in its original form from:
//
//   http://labs.flog.nz.co/raytracer/
//
// It has been modified slightly by Google to work as a standalone
// benchmark, but the all the computational code remains
// untouched. This file also contains a copy of parts of the Prototype
// JavaScript framework which is used by the ray tracer.
//
// It has been further modified by Andrew Morris for use as a benchmark
// in the ValueScript project.

// ------------------------------------------------------------------------
// ------------------------------------------------------------------------

// The code below is based on a concatenation of the following files:
//
//   flog/color.js
//   flog/light.js
//   flog/vector.js
//   flog/ray.js
//   flog/scene.js
//   flog/material/basematerial.js
//   flog/material/solid.js
//   flog/material/chessboard.js
//   flog/shape/baseshape.js
//   flog/shape/sphere.js
//   flog/shape/plane.js
//   flog/intersectioninfo.js
//   flog/camera.js
//   flog/background.js
//   flog/engine.js

class Color {
  red;
  green;
  blue;

  constructor(r = 0, g = 0, b = 0) {
    this.red = r;
    this.green = g;
    this.blue = b;
  }

  static add(c1: Color, c2: Color) {
    return new Color(
      c1.red + c2.red,
      c1.green + c2.green,
      c1.blue + c2.blue,
    );
  }

  static addScalar(c1: Color, s: number) {
    let result = new Color(
      c1.red + s,
      c1.green + s,
      c1.blue + s,
    );

    result.limit();

    return result;
  }

  static subtract(c1: Color, c2: Color) {
    return new Color(
      c1.red - c2.red,
      c1.green - c2.green,
      c1.blue - c2.blue,
    );
  }

  static multiply(c1: Color, c2: Color) {
    return new Color(
      c1.red * c2.red,
      c1.green * c2.green,
      c1.blue * c2.blue,
    );
  }

  static multiplyScalar(c1: Color, f: number) {
    return new Color(
      c1.red * f,
      c1.green * f,
      c1.blue * f,
    );
  }

  static divideFactor(c1: Color, f: number) {
    return new Color(
      c1.red / f,
      c1.green / f,
      c1.blue / f,
    );
  }

  limit() {
    this.red = (this.red > 0.0) ? ((this.red > 1.0) ? 1.0 : this.red) : 0.0;
    this.green = (this.green > 0.0)
      ? ((this.green > 1.0) ? 1.0 : this.green)
      : 0.0;
    this.blue = (this.blue > 0.0) ? ((this.blue > 1.0) ? 1.0 : this.blue) : 0.0;
  }

  distance(color: Color) {
    let d = Math.abs(this.red - color.red) +
      Math.abs(this.green - color.green) + Math.abs(this.blue - color.blue);
    return d;
  }

  static blend(c1: Color, c2: Color, w: number) {
    return Color.add(
      Color.multiplyScalar(c1, 1 - w),
      Color.multiplyScalar(c2, w),
    );
  }

  brightness() {
    let r = Math.floor(this.red * 255);
    let g = Math.floor(this.green * 255);
    let b = Math.floor(this.blue * 255);
    return (r * 77 + g * 150 + b * 29) >> 8;
  }

  toString() {
    let r = Math.floor(this.red * 255);
    let g = Math.floor(this.green * 255);
    let b = Math.floor(this.blue * 255);

    return "rgb(" + r + "," + g + "," + b + ")";
  }
}

class Light {
  position;
  color;
  intensity;

  constructor(pos: Vector, color: Color, intensity = 10) {
    this.position = pos;
    this.color = color;
    this.intensity = intensity;
  }

  toString() {
    return "Light [" + this.position.x + "," + this.position.y + "," +
      this.position.z + "]";
  }
}

class Vector {
  x;
  y;
  z;

  constructor(x = 0, y = 0, z = 0) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  copy(vector: Vector) {
    this.x = vector.x;
    this.y = vector.y;
    this.z = vector.z;
  }

  normalize() {
    let m = this.magnitude();
    return new Vector(this.x / m, this.y / m, this.z / m);
  }

  magnitude() {
    return Math.sqrt((this.x * this.x) + (this.y * this.y) + (this.z * this.z));
  }

  cross(w: Vector) {
    return new Vector(
      -this.z * w.y + this.y * w.z,
      this.z * w.x - this.x * w.z,
      -this.y * w.x + this.x * w.y,
    );
  }

  dot(w: Vector) {
    return this.x * w.x + this.y * w.y + this.z * w.z;
  }

  static add(v: Vector, w: Vector) {
    return new Vector(w.x + v.x, w.y + v.y, w.z + v.z);
  }

  static subtract(v: Vector, w: Vector) {
    if (!w || !v) throw "Vectors must be defined [" + v + "," + w + "]";
    return new Vector(v.x - w.x, v.y - w.y, v.z - w.z);
  }

  static multiplyVector(v: Vector, w: Vector) {
    return new Vector(v.x * w.x, v.y * w.y, v.z * w.z);
  }

  static multiplyScalar(v: Vector, w: number) {
    return new Vector(v.x * w, v.y * w, v.z * w);
  }

  toString() {
    return "Vector [" + this.x + "," + this.y + "," + this.z + "]";
  }
}

class Ray {
  position;
  direction;

  constructor(pos: Vector, dir: Vector) {
    this.position = pos;
    this.direction = dir;
  }

  toString() {
    return "Ray [" + this.position + "," + this.direction + "]";
  }
}

class Scene {
  camera;
  shapes: Shape[];
  lights: Light[];
  background;

  constructor() {
    this.camera = new Camera(
      new Vector(0, 0, -5),
      new Vector(0, 0, 1),
      new Vector(0, 1, 0),
    );
    this.shapes = [];
    this.lights = [];
    this.background = new Background(
      new Color(0, 0, 0.5),
      0.2,
    );
  }
}

type Material = {
  gloss: number;
  transparency: number;
  reflection: number;
  refraction: number;
  hasTexture: boolean;

  getColor(u: number, v: number): Color;
};

function wrapUpMaterial(t: number) {
  t = t % 2.0;
  if (t < -1) t += 2.0;
  if (t >= 1) t -= 2.0;
  return t;
}

class SolidMaterial implements Material {
  reflection: number;
  refraction: number;
  transparency: number;
  gloss: number;
  hasTexture: boolean;

  color;

  constructor(
    color: Color,
    reflection: number,
    _refraction: number,
    transparency: number,
    gloss: number,
  ) {
    this.color = color;
    this.reflection = reflection;
    this.refraction = 0.5; // TODO: Why not use parameter?
    this.transparency = transparency;
    this.gloss = gloss;
    this.hasTexture = false;
  }

  getColor(_u: number, _v: number) {
    return this.color;
  }

  toString() {
    return "SolidMaterial [gloss=" + this.gloss + ", transparency=" +
      this.transparency + ", hasTexture=" + this.hasTexture + "]";
  }
}

class ChessboardMaterial implements Material {
  reflection: number;
  refraction: number;
  transparency: number;
  gloss: number;
  hasTexture: boolean;

  colorEven;
  colorOdd;
  density;

  constructor(
    colorEven: Color,
    colorOdd: Color,
    reflection: number,
    transparency: number,
    gloss: number,
    density: number,
  ) {
    this.colorEven = colorEven;
    this.colorOdd = colorOdd;
    this.reflection = reflection;
    this.refraction = 0.5;
    this.transparency = transparency;
    this.gloss = gloss;
    this.density = density;
    this.hasTexture = true;
  }

  getColor(u: number, v: number) {
    let t = wrapUpMaterial(u * this.density) * wrapUpMaterial(v * this.density);

    if (t < 0.0) {
      return this.colorEven;
    } else {
      return this.colorOdd;
    }
  }

  toString() {
    return "ChessMaterial [gloss=" + this.gloss + ", transparency=" +
      this.transparency + ", hasTexture=" + this.hasTexture + "]";
  }
}

type Shape = {
  material: Material;
  position: Vector;
  intersect(ray: Ray): IntersectionInfo;
  toString(): string;
};

class Sphere implements Shape {
  radius;
  position;
  material;

  constructor(pos: Vector, radius: number, material: Material) {
    this.radius = radius;
    this.position = pos;
    this.material = material;
  }

  intersect(ray: Ray) {
    let info = new IntersectionInfo();
    info.shape = this;

    let dst = Vector.subtract(
      ray.position,
      this.position,
    );

    let B = dst.dot(ray.direction);
    let C = dst.dot(dst) - (this.radius * this.radius);
    let D = (B * B) - C;

    if (D > 0) { // intersection!
      info.isHit = true;
      info.distance = (-B) - Math.sqrt(D);
      info.position = Vector.add(
        ray.position,
        Vector.multiplyScalar(
          ray.direction,
          info.distance,
        ),
      );
      info.normal = Vector.subtract(
        info.position,
        this.position,
      ).normalize();

      info.color = this.material.getColor(0, 0);
    } else {
      info.isHit = false;
    }
    return info;
  }

  toString() {
    return "Sphere [position=" + this.position + ", radius=" + this.radius +
      "]";
  }
}

class Plane implements Shape {
  position;
  d;
  material;

  constructor(pos: Vector, d: number, material: Material) {
    this.position = pos;
    this.d = d;
    this.material = material;
  }

  intersect(ray: Ray) {
    let info = new IntersectionInfo();

    let Vd = this.position.dot(ray.direction);
    if (Vd == 0) return info; // no intersection

    let t = -(this.position.dot(ray.position) + this.d) / Vd;
    if (t <= 0) return info;

    info.shape = this;
    info.isHit = true;
    info.position = Vector.add(
      ray.position,
      Vector.multiplyScalar(
        ray.direction,
        t,
      ),
    );
    info.normal = this.position;
    info.distance = t;

    if (this.material.hasTexture) {
      let vU = new Vector(
        this.position.y,
        this.position.z,
        -this.position.x,
      );
      let vV = vU.cross(this.position);
      let u = info.position.dot(vU);
      let v = info.position.dot(vV);
      info.color = this.material.getColor(u, v);
    } else {
      info.color = this.material.getColor(0, 0);
    }

    return info;
  }

  toString() {
    return "Plane [" + this.position + ", d=" + this.d + "]";
  }
}

class IntersectionInfo {
  isHit = false;
  hitCount = 0;
  shape: Shape | null = null;
  position: Vector | null = null;
  normal: Vector | null = null;
  color;
  distance: number | null = null;

  constructor() {
    this.color = new Color(0, 0, 0);
  }

  toString() {
    return "Intersection [" + this.position + "]";
  }
}

class Camera {
  position;
  lookAt;
  equator;
  up;
  screen;

  constructor(pos: Vector, lookAt: Vector, up: Vector) {
    this.position = pos;
    this.lookAt = lookAt;
    this.up = up;
    this.equator = lookAt.normalize().cross(this.up);
    this.screen = Vector.add(
      this.position,
      this.lookAt,
    );
  }

  getRay(vx: number, vy: number) {
    let pos = Vector.subtract(
      this.screen,
      Vector.subtract(
        Vector.multiplyScalar(this.equator, vx),
        Vector.multiplyScalar(this.up, vy),
      ),
    );
    pos.y = pos.y * -1;
    let dir = Vector.subtract(
      pos,
      this.position,
    );

    let ray = new Ray(pos, dir.normalize());

    return ray;
  }

  toString() {
    return "Ray []";
  }
}

class Background {
  color;
  ambience;

  constructor(color: Color, ambience: number) {
    this.color = color;
    this.ambience = ambience;
  }
}

type EngineOptions = {
  canvasWidth: number;
  canvasHeight: number;
  pixelWidth: number;
  pixelHeight: number;
  renderDiffuse: boolean;
  renderHighlights: boolean;
  renderShadows: boolean;
  renderReflections: boolean;
  rayDepth: number;
};

class Engine {
  canvas: unknown = null; /* 2d context we can render to */
  options;

  // Variable used to hold a number that can be used to verify that
  // the scene was ray traced correctly.
  checkNumber = 0;

  constructor(options: EngineOptions) {
    this.options = options;

    this.options.canvasHeight /= this.options.pixelHeight;
    this.options.canvasWidth /= this.options.pixelWidth;

    /* TODO: dynamically include other scripts */
  }

  setPixel(x: number, y: number, color: Color) {
    let _pxW, _pxH;
    _pxW = this.options.pixelWidth;
    _pxH = this.options.pixelHeight;

    if (this.canvas) {
      throw new Error("Not implemented: canvas");
      // this.canvas.fillStyle = color.toString();
      // this.canvas.fillRect(x * pxW, y * pxH, pxW, pxH);
    } else {
      if (x === y) {
        this.checkNumber += color.brightness();
      }
      // print(x * pxW, y * pxH, pxW, pxH);
    }
  }

  renderScene(scene: Scene, canvas: unknown) {
    this.checkNumber = 0;
    /* Get canvas */
    if (canvas) {
      throw new Error("Not implemented: canvas");
      // this.canvas = canvas.getContext("2d");
    } else {
      this.canvas = null;
    }

    let canvasHeight = this.options.canvasHeight;
    let canvasWidth = this.options.canvasWidth;

    for (let y = 0; y < canvasHeight; y++) {
      for (let x = 0; x < canvasWidth; x++) {
        let yp = y * 1.0 / canvasHeight * 2 - 1;
        let xp = x * 1.0 / canvasWidth * 2 - 1;

        let ray = scene.camera.getRay(xp, yp);

        let color = this.getPixelColor(ray, scene);

        this.setPixel(x, y, color);
      }
    }
    if (this.checkNumber !== 2321) {
      throw new Error("Scene rendered incorrectly");
    }
  }

  getPixelColor(ray: Ray, scene: Scene) {
    let info = this.testIntersection(ray, scene, null);
    if (info.isHit) {
      let color = this.rayTrace(info, ray, scene, 0);
      return color;
    }
    return scene.background.color;
  }

  testIntersection(ray: Ray, scene: Scene, exclude: Shape | null) {
    let hits = 0;
    let best = new IntersectionInfo();
    best.distance = 2000;

    for (let i = 0; i < scene.shapes.length; i++) {
      let shape = scene.shapes[i];

      if (shape != exclude) {
        let info = shape.intersect(ray);
        if (
          info.isHit && info.distance! >= 0 && info.distance! < best.distance!
        ) {
          best = info;
          hits++;
        }
      }
    }
    best.hitCount = hits;
    return best;
  }

  getReflectionRay(P: Vector, N: Vector, V: Vector) {
    let c1 = -N.dot(V);
    let R1 = Vector.add(
      Vector.multiplyScalar(N, 2 * c1),
      V,
    );
    return new Ray(P, R1);
  }

  rayTrace(info: IntersectionInfo, ray: Ray, scene: Scene, depth: number) {
    // Calc ambient
    let color = Color.multiplyScalar(
      info.color,
      scene.background.ambience,
    );
    let _oldColor = color;
    let shininess = Math.pow(10, info.shape!.material.gloss + 1);

    for (let i = 0; i < scene.lights.length; i++) {
      let light = scene.lights[i];

      // Calc diffuse lighting
      let v = Vector.subtract(
        light.position,
        info.position!,
      ).normalize();

      if (this.options.renderDiffuse) {
        let L = v.dot(info.normal!);
        if (L > 0.0) {
          color = Color.add(
            color,
            Color.multiply(
              info.color,
              Color.multiplyScalar(
                light.color,
                L,
              ),
            ),
          );
        }
      }

      // The greater the depth the more accurate the colours, but
      // this is exponentially (!) expensive
      if (depth <= this.options.rayDepth) {
        // calculate reflection ray
        if (
          this.options.renderReflections && info.shape!.material.reflection > 0
        ) {
          let reflectionRay = this.getReflectionRay(
            info.position!,
            info.normal!,
            ray.direction,
          );
          let refl = this.testIntersection(reflectionRay, scene, info.shape);

          if (refl.isHit && refl.distance! > 0) {
            refl.color = this.rayTrace(refl, reflectionRay, scene, depth + 1);
          } else {
            refl.color = scene.background.color;
          }

          color = Color.blend(
            color,
            refl.color,
            info.shape!.material.reflection,
          );
        }

        // Refraction
        /* TODO */
      }

      /* Render shadows and highlights */

      let shadowInfo = new IntersectionInfo();

      if (this.options.renderShadows) {
        let shadowRay = new Ray(info.position!, v);

        shadowInfo = this.testIntersection(shadowRay, scene, info.shape);
        if (
          shadowInfo.isHit &&
          shadowInfo.shape != info.shape /*&& shadowInfo.shape.type != 'PLANE'*/
        ) {
          let vA = Color.multiplyScalar(color, 0.5);
          let dB = 0.5 * Math.pow(shadowInfo.shape!.material.transparency, 0.5);
          color = Color.addScalar(vA, dB);
        }
      }

      // Phong specular highlights
      if (
        this.options.renderHighlights && !shadowInfo.isHit &&
        info.shape!.material.gloss > 0
      ) {
        let Lv = Vector.subtract(
          info.shape!.position,
          light.position,
        ).normalize();

        let E = Vector.subtract(
          scene.camera.position,
          info.shape!.position,
        ).normalize();

        let H = Vector.subtract(
          E,
          Lv,
        ).normalize();

        let glossWeight = Math.pow(Math.max(info.normal!.dot(H), 0), shininess);
        color = Color.add(
          Color.multiplyScalar(
            light.color,
            glossWeight,
          ),
          color,
        );
      }
    }
    color.limit();
    return color;
  }
}

export default function renderScene() {
  let scene = new Scene();

  scene.camera = new Camera(
    new Vector(0, 0, -15),
    new Vector(-0.2, 0, 5),
    new Vector(0, 1, 0),
  );

  scene.background = new Background(
    new Color(0.5, 0.5, 0.5),
    0.4,
  );

  let sphere = new Sphere(
    new Vector(-1.5, 1.5, 2),
    1.5,
    new SolidMaterial(
      new Color(0, 0.5, 0.5),
      0.3,
      0.0,
      0.0,
      2.0,
    ),
  );

  let sphere1 = new Sphere(
    new Vector(1, 0.25, 1),
    0.5,
    new SolidMaterial(
      new Color(0.9, 0.9, 0.9),
      0.1,
      0.0,
      0.0,
      1.5,
    ),
  );

  let plane = new Plane(
    new Vector(0.1, 0.9, -0.5).normalize(),
    1.2,
    new ChessboardMaterial(
      new Color(1, 1, 1),
      new Color(0, 0, 0),
      0.2,
      0.0,
      1.0,
      0.7,
    ),
  );

  scene.shapes.push(plane);
  scene.shapes.push(sphere);
  scene.shapes.push(sphere1);

  let light = new Light(
    new Vector(5, 10, -1),
    new Color(0.8, 0.8, 0.8),
  );

  let light1 = new Light(
    new Vector(-3, 5, -15),
    new Color(0.8, 0.8, 0.8),
    100,
  );

  scene.lights.push(light);
  scene.lights.push(light1);

  let imageWidth = 100; // $F('imageWidth');
  let imageHeight = 100; // $F('imageHeight');
  let pixelSize = [5, 5]; //  $F('pixelSize').split(',');
  let renderDiffuse = true; // $F('renderDiffuse');
  let renderShadows = true; // $F('renderShadows');
  let renderHighlights = true; // $F('renderHighlights');
  let renderReflections = true; // $F('renderReflections');
  let rayDepth = 2; //$F('rayDepth');

  let raytracer = new Engine(
    {
      canvasWidth: imageWidth,
      canvasHeight: imageHeight,
      pixelWidth: pixelSize[0],
      pixelHeight: pixelSize[1],
      "renderDiffuse": renderDiffuse,
      "renderHighlights": renderHighlights,
      "renderShadows": renderShadows,
      "renderReflections": renderReflections,
      "rayDepth": rayDepth,
    },
  );

  raytracer.renderScene(scene, null /* , 0 */);
}
