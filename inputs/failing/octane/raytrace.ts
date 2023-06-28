// The ray tracer code in this file is written by Adam Burmister. It
// is available in its original form from:
//
//   http://labs.flog.nz.co/raytracer/
//
// It has been modified slightly by Google to work as a standalone
// benchmark, but the all the computational code remains
// untouched. This file also contains a copy of parts of the Prototype
// JavaScript framework which is used by the ray tracer.

// Variable used to hold a number that can be used to verify that
// the scene was ray traced correctly.
let checkNumber;

// ------------------------------------------------------------------------
// ------------------------------------------------------------------------

// The rest of this file is the actual ray tracer written by Adam
// Burmister. It's a concatenation of the following files:
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

  constructor(pos: Vector, color: Color, intensity: number) {
    this.position = pos;
    this.color = color;
    this.intensity = intensity ? intensity : 10.0;
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
  shapes;
  lights;
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

class Shape {
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

class Sphere {
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

/* Fake a Flog.* namespace */
if (typeof (Flog.RayTracer) == "undefined") Flog.RayTracer = {};
if (typeof (Flog.RayTracer.Shape) == "undefined") Flog.RayTracer.Shape = {};

Flog.RayTracer.Shape.Plane = Class.create();

Flog.RayTracer.Shape.Plane.prototype = {
  d: 0.0,

  initialize: function (pos, d, material) {
    this.position = pos;
    this.d = d;
    this.material = material;
  },

  intersect: function (ray) {
    let info = new Flog.RayTracer.IntersectionInfo();

    let Vd = this.position.dot(ray.direction);
    if (Vd == 0) return info; // no intersection

    let t = -(this.position.dot(ray.position) + this.d) / Vd;
    if (t <= 0) return info;

    info.shape = this;
    info.isHit = true;
    info.position = Flog.RayTracer.Vector.prototype.add(
      ray.position,
      Flog.RayTracer.Vector.prototype.multiplyScalar(
        ray.direction,
        t,
      ),
    );
    info.normal = this.position;
    info.distance = t;

    if (this.material.hasTexture) {
      let vU = new Flog.RayTracer.Vector(
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
  },

  toString: function () {
    return "Plane [" + this.position + ", d=" + this.d + "]";
  },
};
/* Fake a Flog.* namespace */
if (typeof (Flog.RayTracer) == "undefined") Flog.RayTracer = {};

Flog.RayTracer.IntersectionInfo = Class.create();

Flog.RayTracer.IntersectionInfo.prototype = {
  isHit: false,
  hitCount: 0,
  shape: null,
  position: null,
  normal: null,
  color: null,
  distance: null,

  initialize: function () {
    this.color = new Flog.RayTracer.Color(0, 0, 0);
  },

  toString: function () {
    return "Intersection [" + this.position + "]";
  },
};
/* Fake a Flog.* namespace */
if (typeof (Flog.RayTracer) == "undefined") Flog.RayTracer = {};

Flog.RayTracer.Camera = Class.create();

Flog.RayTracer.Camera.prototype = {
  position: null,
  lookAt: null,
  equator: null,
  up: null,
  screen: null,

  initialize: function (pos, lookAt, up) {
    this.position = pos;
    this.lookAt = lookAt;
    this.up = up;
    this.equator = lookAt.normalize().cross(this.up);
    this.screen = Flog.RayTracer.Vector.prototype.add(
      this.position,
      this.lookAt,
    );
  },

  getRay: function (vx, vy) {
    let pos = Flog.RayTracer.Vector.prototype.subtract(
      this.screen,
      Flog.RayTracer.Vector.prototype.subtract(
        Flog.RayTracer.Vector.prototype.multiplyScalar(this.equator, vx),
        Flog.RayTracer.Vector.prototype.multiplyScalar(this.up, vy),
      ),
    );
    pos.y = pos.y * -1;
    let dir = Flog.RayTracer.Vector.prototype.subtract(
      pos,
      this.position,
    );

    let ray = new Flog.RayTracer.Ray(pos, dir.normalize());

    return ray;
  },

  toString: function () {
    return "Ray []";
  },
};
/* Fake a Flog.* namespace */
if (typeof (Flog.RayTracer) == "undefined") Flog.RayTracer = {};

Flog.RayTracer.Background = Class.create();

Flog.RayTracer.Background.prototype = {
  color: null,
  ambience: 0.0,

  initialize: function (color, ambience) {
    this.color = color;
    this.ambience = ambience;
  },
};
/* Fake a Flog.* namespace */
if (typeof (Flog.RayTracer) == "undefined") Flog.RayTracer = {};

Flog.RayTracer.Engine = Class.create();

Flog.RayTracer.Engine.prototype = {
  canvas: null, /* 2d context we can render to */

  initialize: function (options) {
    this.options = Object.extend({
      canvasHeight: 100,
      canvasWidth: 100,
      pixelWidth: 2,
      pixelHeight: 2,
      renderDiffuse: false,
      renderShadows: false,
      renderHighlights: false,
      renderReflections: false,
      rayDepth: 2,
    }, options || {});

    this.options.canvasHeight /= this.options.pixelHeight;
    this.options.canvasWidth /= this.options.pixelWidth;

    /* TODO: dynamically include other scripts */
  },

  setPixel: function (x, y, color) {
    let pxW, pxH;
    pxW = this.options.pixelWidth;
    pxH = this.options.pixelHeight;

    if (this.canvas) {
      this.canvas.fillStyle = color.toString();
      this.canvas.fillRect(x * pxW, y * pxH, pxW, pxH);
    } else {
      if (x === y) {
        checkNumber += color.brightness();
      }
      // print(x * pxW, y * pxH, pxW, pxH);
    }
  },

  renderScene: function (scene, canvas) {
    checkNumber = 0;
    /* Get canvas */
    if (canvas) {
      this.canvas = canvas.getContext("2d");
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
    if (checkNumber !== 2321) {
      throw new Error("Scene rendered incorrectly");
    }
  },

  getPixelColor: function (ray, scene) {
    let info = this.testIntersection(ray, scene, null);
    if (info.isHit) {
      let color = this.rayTrace(info, ray, scene, 0);
      return color;
    }
    return scene.background.color;
  },

  testIntersection: function (ray, scene, exclude) {
    let hits = 0;
    let best = new Flog.RayTracer.IntersectionInfo();
    best.distance = 2000;

    for (let i = 0; i < scene.shapes.length; i++) {
      let shape = scene.shapes[i];

      if (shape != exclude) {
        let info = shape.intersect(ray);
        if (info.isHit && info.distance >= 0 && info.distance < best.distance) {
          best = info;
          hits++;
        }
      }
    }
    best.hitCount = hits;
    return best;
  },

  getReflectionRay: function (P, N, V) {
    let c1 = -N.dot(V);
    let R1 = Flog.RayTracer.Vector.prototype.add(
      Flog.RayTracer.Vector.prototype.multiplyScalar(N, 2 * c1),
      V,
    );
    return new Flog.RayTracer.Ray(P, R1);
  },

  rayTrace: function (info, ray, scene, depth) {
    // Calc ambient
    let color = Flog.RayTracer.Color.prototype.multiplyScalar(
      info.color,
      scene.background.ambience,
    );
    let oldColor = color;
    let shininess = Math.pow(10, info.shape.material.gloss + 1);

    for (let i = 0; i < scene.lights.length; i++) {
      let light = scene.lights[i];

      // Calc diffuse lighting
      let v = Flog.RayTracer.Vector.prototype.subtract(
        light.position,
        info.position,
      ).normalize();

      if (this.options.renderDiffuse) {
        let L = v.dot(info.normal);
        if (L > 0.0) {
          color = Flog.RayTracer.Color.prototype.add(
            color,
            Flog.RayTracer.Color.prototype.multiply(
              info.color,
              Flog.RayTracer.Color.prototype.multiplyScalar(
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
          this.options.renderReflections && info.shape.material.reflection > 0
        ) {
          let reflectionRay = this.getReflectionRay(
            info.position,
            info.normal,
            ray.direction,
          );
          let refl = this.testIntersection(reflectionRay, scene, info.shape);

          if (refl.isHit && refl.distance > 0) {
            refl.color = this.rayTrace(refl, reflectionRay, scene, depth + 1);
          } else {
            refl.color = scene.background.color;
          }

          color = Flog.RayTracer.Color.prototype.blend(
            color,
            refl.color,
            info.shape.material.reflection,
          );
        }

        // Refraction
        /* TODO */
      }

      /* Render shadows and highlights */

      let shadowInfo = new Flog.RayTracer.IntersectionInfo();

      if (this.options.renderShadows) {
        let shadowRay = new Flog.RayTracer.Ray(info.position, v);

        shadowInfo = this.testIntersection(shadowRay, scene, info.shape);
        if (
          shadowInfo.isHit &&
          shadowInfo.shape != info.shape /*&& shadowInfo.shape.type != 'PLANE'*/
        ) {
          let vA = Flog.RayTracer.Color.prototype.multiplyScalar(color, 0.5);
          let dB = 0.5 * Math.pow(shadowInfo.shape.material.transparency, 0.5);
          color = Flog.RayTracer.Color.prototype.addScalar(vA, dB);
        }
      }

      // Phong specular highlights
      if (
        this.options.renderHighlights && !shadowInfo.isHit &&
        info.shape.material.gloss > 0
      ) {
        let Lv = Flog.RayTracer.Vector.prototype.subtract(
          info.shape.position,
          light.position,
        ).normalize();

        let E = Flog.RayTracer.Vector.prototype.subtract(
          scene.camera.position,
          info.shape.position,
        ).normalize();

        let H = Flog.RayTracer.Vector.prototype.subtract(
          E,
          Lv,
        ).normalize();

        let glossWeight = Math.pow(Math.max(info.normal.dot(H), 0), shininess);
        color = Flog.RayTracer.Color.prototype.add(
          Flog.RayTracer.Color.prototype.multiplyScalar(
            light.color,
            glossWeight,
          ),
          color,
        );
      }
    }
    color.limit();
    return color;
  },
};

export default function renderScene() {
  let scene = new Flog.RayTracer.Scene();

  scene.camera = new Flog.RayTracer.Camera(
    new Flog.RayTracer.Vector(0, 0, -15),
    new Flog.RayTracer.Vector(-0.2, 0, 5),
    new Flog.RayTracer.Vector(0, 1, 0),
  );

  scene.background = new Flog.RayTracer.Background(
    new Flog.RayTracer.Color(0.5, 0.5, 0.5),
    0.4,
  );

  let sphere = new Flog.RayTracer.Shape.Sphere(
    new Flog.RayTracer.Vector(-1.5, 1.5, 2),
    1.5,
    new Flog.RayTracer.Material.Solid(
      new Flog.RayTracer.Color(0, 0.5, 0.5),
      0.3,
      0.0,
      0.0,
      2.0,
    ),
  );

  let sphere1 = new Flog.RayTracer.Shape.Sphere(
    new Flog.RayTracer.Vector(1, 0.25, 1),
    0.5,
    new Flog.RayTracer.Material.Solid(
      new Flog.RayTracer.Color(0.9, 0.9, 0.9),
      0.1,
      0.0,
      0.0,
      1.5,
    ),
  );

  let plane = new Flog.RayTracer.Shape.Plane(
    new Flog.RayTracer.Vector(0.1, 0.9, -0.5).normalize(),
    1.2,
    new Flog.RayTracer.Material.Chessboard(
      new Flog.RayTracer.Color(1, 1, 1),
      new Flog.RayTracer.Color(0, 0, 0),
      0.2,
      0.0,
      1.0,
      0.7,
    ),
  );

  scene.shapes.push(plane);
  scene.shapes.push(sphere);
  scene.shapes.push(sphere1);

  let light = new Flog.RayTracer.Light(
    new Flog.RayTracer.Vector(5, 10, -1),
    new Flog.RayTracer.Color(0.8, 0.8, 0.8),
  );

  let light1 = new Flog.RayTracer.Light(
    new Flog.RayTracer.Vector(-3, 5, -15),
    new Flog.RayTracer.Color(0.8, 0.8, 0.8),
    100,
  );

  scene.lights.push(light);
  scene.lights.push(light1);

  let imageWidth = 100; // $F('imageWidth');
  let imageHeight = 100; // $F('imageHeight');
  let pixelSize = "5,5".split(","); //  $F('pixelSize').split(',');
  let renderDiffuse = true; // $F('renderDiffuse');
  let renderShadows = true; // $F('renderShadows');
  let renderHighlights = true; // $F('renderHighlights');
  let renderReflections = true; // $F('renderReflections');
  let rayDepth = 2; //$F('rayDepth');

  let raytracer = new Flog.RayTracer.Engine(
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

  raytracer.renderScene(scene, null, 0);
}

renderScene();
