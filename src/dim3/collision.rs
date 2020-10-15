/// # Resources
///
/// Inspired by Randy Gaul's qu3e engine
/// [https://github.com/RandyGaul/qu3e/blob/master/src/collision/q3Collide.cpp](qu3e/q3Collide.cpp)
use bevy::math::*;
use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};

use super::*;

trait Mult {
    fn mult(&self, v: Vec3) -> Vec3;
}

impl Mult for Transform {
    fn mult(&self, v: Vec3) -> Vec3 {
        self.rotation().conjugate() * (v - self.translation())
    }
}

impl Mult for Quat {
    fn mult(&self, v: Vec3) -> Vec3 {
        self.conjugate() * v
    }
}

impl Mult for Mat3 {
    fn mult(&self, v: Vec3) -> Vec3 {
        self.transpose() * v
    }
}

trait Mat3Ext {
    fn column0(&self) -> Vec3;
    fn column1(&self) -> Vec3;
    fn column2(&self) -> Vec3;
    fn row0(&self) -> Vec3;
    fn row1(&self) -> Vec3;
    fn row2(&self) -> Vec3;
}

impl Mat3Ext for Mat3 {
    fn column0(&self) -> Vec3 {
        Vec3::from(self.to_cols_array_2d()[0])
    }

    fn column1(&self) -> Vec3 {
        Vec3::from(self.to_cols_array_2d()[1])
    }

    fn column2(&self) -> Vec3 {
        Vec3::from(self.to_cols_array_2d()[2])
    }

    fn row0(&self) -> Vec3 {
        self.transpose().column0()
    }

    fn row1(&self) -> Vec3 {
        self.transpose().column1()
    }

    fn row2(&self) -> Vec3 {
        self.transpose().column2()
    }
}

trait Mat4Ext {
    fn truncate(&self) -> Mat3;
}

impl Mat4Ext for Mat4 {
    fn truncate(&self) -> Mat3 {
        Mat3::from_cols(
            self.x_axis().truncate().into(),
            self.y_axis().truncate().into(),
            self.z_axis().truncate().into(),
        )
    }
}

enum TrackFaceAxis {
    None,
    Some { axis: u32, max: f32, normal: Vec3 },
    Yes,
}

fn track_face_axis(n: u32, s: f32, smax: f32, normal: Vec3) -> TrackFaceAxis {
    if s > 0.0 {
        return TrackFaceAxis::None;
    }

    if s > smax {
        let max = s;
        let axis = n;
        return TrackFaceAxis::Some { max, axis, normal };
    }

    TrackFaceAxis::Yes
}

enum TrackEdgeAxis {
    None,
    Some { axis: u32, max: f32, normal: Vec3 },
    Yes,
}

fn track_edge_axis(n: u32, mut s: f32, smax: f32, normal: Vec3) -> TrackEdgeAxis {
    if s > 0.0 {
        return TrackEdgeAxis::None;
    }

    let l = normal.length_reciprocal();
    s *= l;

    if s > smax {
        let max = s;
        let axis = n;
        let normal = normal * l;
        return TrackEdgeAxis::Some { max, axis, normal };
    }

    TrackEdgeAxis::Yes
}

#[derive(Debug, Clone, Copy)]
struct FeaturePair {
    inr: u8,
    outr: u8,
    ini: u8,
    outi: u8,
}

impl Default for FeaturePair {
    fn default() -> Self {
        Self {
            inr: u8::MAX,
            outr: u8::MAX,
            ini: u8::MAX,
            outi: u8::MAX,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct ClipVertex {
    v: Vec3,
    f: FeaturePair,
}

fn compute_incident_face(itx: Transform, e: Vec3, n: Vec3) -> [ClipVertex; 4] {
    let n = -itx.rotation().mult(n);
    let absn = n.abs();

    if absn.x() > absn.y() && absn.x() > absn.z() {
        if n.x() > 0.0 {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 9,
                        outi: 1,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx.value().transform_point3(Vec3::new(e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 1,
                        outi: 8,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 8,
                        outi: 7,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 7,
                        outi: 9,
                        ..Default::default()
                    },
                },
            ]
        } else {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 5,
                        outi: 11,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 11,
                        outi: 3,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 3,
                        outi: 10,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 10,
                        outi: 5,
                        ..Default::default()
                    },
                },
            ]
        }
    } else if absn.y() > absn.x() && absn.y() > absn.z() {
        if n.y() > 0.0 {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 3,
                        outi: 0,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx.value().transform_point3(Vec3::new(e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 0,
                        outi: 1,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 1,
                        outi: 2,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 2,
                        outi: 3,
                        ..Default::default()
                    },
                },
            ]
        } else {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 7,
                        outi: 4,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 4,
                        outi: 5,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 5,
                        outi: 6,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 6,
                        outi: 7,
                        ..Default::default()
                    },
                },
            ]
        }
    } else {
        if n.z() > 0.0 {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 0,
                        outi: 11,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 11,
                        outi: 4,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), e.z())),
                    f: FeaturePair {
                        ini: 4,
                        outi: 8,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx.value().transform_point3(Vec3::new(e.x(), e.y(), e.z())),
                    f: FeaturePair {
                        ini: 8,
                        outi: 0,
                        ..Default::default()
                    },
                },
            ]
        } else {
            [
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 9,
                        outi: 6,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), -e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 6,
                        outi: 10,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(-e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 10,
                        outi: 2,
                        ..Default::default()
                    },
                },
                ClipVertex {
                    v: itx
                        .value()
                        .transform_point3(Vec3::new(e.x(), e.y(), -e.z())),
                    f: FeaturePair {
                        ini: 2,
                        outi: 9,
                        ..Default::default()
                    },
                },
            ]
        }
    }
}

struct RefEb {
    clip_edges: [u8; 4],
    basis: Mat3,
    e: Vec3,
}

fn compute_reference_edges_and_basis(er: Vec3, rtx: Transform, n: Vec3, mut axis: u32) -> RefEb {
    let n = rtx.rotation().mult(n);

    if axis >= 3 {
        axis -= 3;
    }

    let row_rot = Mat3::from_quat(rtx.rotation()).transpose();
    match axis {
        0 => {
            if n.x() > 0.0 {
                let clip_edges = [1, 8, 7, 9];
                let e = Vec3::new(er.y(), er.z(), er.x());
                let basis =
                    Mat3::from_cols(row_rot.column1(), row_rot.column2(), row_rot.column0())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            } else {
                let clip_edges = [11, 3, 10, 5];
                let e = Vec3::new(er.z(), er.y(), er.x());
                let basis =
                    Mat3::from_cols(row_rot.column2(), row_rot.column1(), -row_rot.column0())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            }
        }
        1 => {
            if n.y() > 0.0 {
                let clip_edges = [0, 1, 2, 3];
                let e = Vec3::new(er.z(), er.x(), er.y());
                let basis =
                    Mat3::from_cols(row_rot.column2(), row_rot.column0(), row_rot.column1())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            } else {
                let clip_edges = [4, 5, 6, 7];
                let e = Vec3::new(er.z(), er.x(), er.y());
                let basis =
                    Mat3::from_cols(row_rot.column2(), -row_rot.column0(), -row_rot.column1())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            }
        }
        2 => {
            if n.z() > 0.0 {
                let clip_edges = [11, 4, 8, 0];
                let e = Vec3::new(er.y(), er.x(), er.z());
                let basis =
                    Mat3::from_cols(-row_rot.column1(), row_rot.column0(), row_rot.column2())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            } else {
                let clip_edges = [6, 10, 2, 9];
                let e = Vec3::new(er.y(), er.x(), er.z());
                let basis =
                    Mat3::from_cols(-row_rot.column1(), -row_rot.column0(), -row_rot.column2())
                        .transpose();
                RefEb {
                    clip_edges,
                    basis,
                    e,
                }
            }
        }
        _ => unimplemented!(),
    }
}

fn orthographic(
    sign: f32,
    e: f32,
    axis: u32,
    clip_edge: u8,
    vin: &[ClipVertex],
) -> SmallVec<[ClipVertex; 8]> {
    fn in_front(a: f32) -> bool {
        a < 0.0
    }

    fn behind(a: f32) -> bool {
        a >= 0.0
    }

    fn on(a: f32) -> bool {
        a < 0.005 && a > -0.005
    }

    let mut out = SmallVec::new();
    let mut a = vin.last().unwrap();

    for b in vin {
        let da = sign * a.v[axis as usize] - e;
        let db = sign * b.v[axis as usize] - e;

        let mut cv = ClipVertex::default();

        if in_front(da) && in_front(db) || on(da) || on(db) {
            debug_assert!(out.len() < 8);
            out.push(*b);
        } else if in_front(da) && behind(db) {
            cv.f = b.f;
            cv.v = a.v + (b.v - a.v) * (da / (da - db));
            cv.f.outr = clip_edge;
            cv.f.outi = 0;
            debug_assert!(out.len() < 8);
            out.push(cv);
        // } else if behind(da) && behind(db) {
        // Randy Gaul's code uses this, but this seems to give incorrect results question mark
        // I probably have a bug somewhere though.
        // NOTE: If you ever encounter clipping inconsistencies, just swap these lines
        } else if behind(da) && in_front(db) {
            cv.f = a.f;
            cv.v = a.v + (b.v - a.v) * (da / (da - db));
            cv.f.inr = clip_edge;
            cv.f.ini = 0;
            debug_assert!(out.len() < 8);
            out.push(cv);

            debug_assert!(out.len() < 8);
            out.push(*b);
        }

        a = b;
    }

    out
}

#[derive(Default)]
struct Clip {
    out: SmallVec<[(ClipVertex, f32); 8]>,
}

fn clip(rpos: Vec3, e: Vec3, clip_edges: [u8; 4], basis: Mat3, incident: [ClipVertex; 4]) -> Clip {
    let mut vin = SmallVec::<[_; 8]>::new();
    let mut vout;

    for inc in &incident {
        vin.push(ClipVertex {
            v: basis.mult(inc.v - rpos),
            f: inc.f,
        });
    }

    vout = orthographic(1.0, e.x(), 0, clip_edges[0], vin.as_slice());

    if vout.is_empty() {
        return Clip::default();
    }

    vin = orthographic(1.0, e.y(), 1, clip_edges[1], vout.as_slice());

    if vin.is_empty() {
        return Clip::default();
    }

    vout = orthographic(-1.0, e.x(), 0, clip_edges[2], vin.as_slice());

    if vout.is_empty() {
        return Clip::default();
    }

    vin = orthographic(-1.0, e.y(), 1, clip_edges[3], vout.as_slice());

    let mut clipped = SmallVec::new();

    for cv in vin {
        let d = cv.v.z() - e.z();

        if d <= 0.0 {
            let vertex = ClipVertex {
                v: basis * cv.v + rpos,
                f: cv.f,
            };

            clipped.push((vertex, d));
        }
    }

    debug_assert!(clipped.len() <= 8);

    Clip { out: clipped }
}

fn edges_contact(pa: Vec3, qa: Vec3, pb: Vec3, qb: Vec3) -> [Vec3; 2] {
    let da = qa - pa;
    let db = qb - pb;
    let r = pa - pb;
    let a = da.dot(da);
    let e = db.dot(db);
    let f = db.dot(r);
    let c = da.dot(r);

    let b = da.dot(db);
    let denom = a * e - b * b;

    let ta = (b * f - c * e) / denom;
    let tb = (b * ta + f) / e;

    [pa + da * ta, pb + db * tb]
}

fn support_edge(tx: Transform, e: Vec3, n: Vec3) -> [Vec3; 2] {
    let n = tx.rotation().mult(n);
    let absn = n.abs();
    let a;
    let b;

    if absn.x() > absn.y() {
        if absn.y() > absn.z() {
            a = Vec3::new(e.x(), e.y(), e.z());
            b = Vec3::new(e.x(), e.y(), -e.z());
        } else {
            a = Vec3::new(e.x(), e.y(), e.z());
            b = Vec3::new(e.x(), -e.y(), e.z());
        }
    } else {
        if absn.x() > absn.z() {
            a = Vec3::new(e.x(), e.y(), e.z());
            b = Vec3::new(e.x(), e.y(), -e.z());
        } else {
            a = Vec3::new(e.x(), e.y(), e.z());
            b = Vec3::new(-e.x(), e.y(), e.z());
        }
    }

    let sign = n.sign();

    let a = a * sign;
    let b = b * sign;

    [
        tx.value().transform_point3(a),
        tx.value().transform_point3(b),
    ]
}

pub fn box_to_box(a: &Obb, b: &Obb) -> Option<Manifold> {
    let mut atx = a.transform;
    let mut btx = b.transform;
    let al = a.local;
    let bl = b.local;
    *atx.value_mut() = *atx.value() * *al.value();
    *btx.value_mut() = *btx.value() * *bl.value();

    let ea = a.extent;
    let eb = b.extent;

    // conjugate is the same as inverse for unit squaternions,
    // inverse is the same as transpose for rotation matrices
    let c = Mat3::from_quat(atx.rotation().conjugate() * btx.rotation());
    let ca = c.to_cols_array_2d();

    let mut absc = [[0.0; 3]; 3];
    let mut parallel = false;
    const EPS: f32 = 1.0_e-6;
    for i in 0..3 {
        for j in 0..3 {
            let val = ca[i][j].abs();
            absc[i][j] = val;

            if val + EPS >= 1.0 {
                parallel = true
            }
        }
    }

    let absca = absc;
    let absc = Mat3::from_cols_array_2d(&absca);

    let t = atx.rotation().mult(btx.translation() - atx.translation());

    let mut s;
    let mut amax = f32::MIN;
    let mut bmax = f32::MIN;
    let mut emax = f32::MIN;
    let mut aaxis = u32::MAX;
    let mut baxis = u32::MAX;
    let mut eaxis = u32::MAX;
    let mut na = Vec3::zero();
    let mut nb = Vec3::zero();
    let mut ne = Vec3::zero();

    let atxr = atx.value().truncate();

    s = t.x().abs() - (ea.x() + absc.column0().dot(eb));
    match track_face_axis(0, s, amax, atxr.row0()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            amax = max;
            aaxis = axis;
            na = normal;
        }
        _ => {}
    }

    s = t.y().abs() - (ea.y() + absc.column1().dot(eb));
    match track_face_axis(1, s, amax, atxr.row1()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            amax = max;
            aaxis = axis;
            na = normal;
        }
        _ => {}
    }

    s = t.z().abs() - (ea.z() + absc.column2().dot(eb));
    match track_face_axis(2, s, amax, atxr.row2()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            amax = max;
            aaxis = axis;
            na = normal;
        }
        _ => {}
    }

    let btxr = btx.value().truncate();

    s = t.dot(c.row0()).abs() - (eb.x() + absc.row0().dot(ea));
    match track_face_axis(3, s, bmax, btxr.row0()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            bmax = max;
            baxis = axis;
            nb = normal;
        }
        _ => {}
    }

    s = t.dot(c.row1()).abs() - (eb.y() + absc.row1().dot(ea));
    match track_face_axis(4, s, bmax, btxr.row1()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            bmax = max;
            baxis = axis;
            nb = normal;
        }
        _ => {}
    }

    s = t.dot(c.row2()).abs() - (eb.z() + absc.row2().dot(ea));
    match track_face_axis(5, s, bmax, btxr.row2()) {
        TrackFaceAxis::None => return None,
        TrackFaceAxis::Some { max, axis, normal } => {
            bmax = max;
            baxis = axis;
            nb = normal;
        }
        _ => {}
    }

    if !parallel {
        let mut ra;
        let mut rb;

        ra = ea.y() * absca[2][0] + ea.z() * absca[1][0];
        rb = eb.y() * absca[0][2] + eb.z() * absca[0][1];
        s = (t.z() * ca[1][0] - t.y() * ca[2][0]).abs() - (ra + rb);
        let normal = Vec3::new(0.0, -ca[2][0], ca[1][0]);
        match track_edge_axis(6, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.y() * absca[2][1] + ea.z() * absca[1][1];
        rb = eb.x() * absca[0][2] + eb.z() * absca[0][0];
        s = (t.z() * ca[1][1] - t.y() * ca[2][1]).abs() - (ra + rb);
        let normal = Vec3::new(0.0, -ca[2][1], ca[1][1]);
        match track_edge_axis(7, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.y() * absca[2][2] + ea.z() * absca[1][2];
        rb = eb.x() * absca[0][1] + eb.y() * absca[0][0];
        s = (t.z() * ca[1][2] - t.y() * ca[2][2]).abs() - (ra + rb);
        let normal = Vec3::new(0.0, -ca[2][2], ca[1][2]);
        match track_edge_axis(8, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[2][0] + ea.z() * absca[0][0];
        rb = eb.y() * absca[1][2] + eb.z() * absca[1][1];
        s = (t.x() * ca[2][0] - t.z() * ca[0][0]).abs() - (ra + rb);
        let normal = Vec3::new(ca[2][0], 0.0, -ca[0][0]);
        match track_edge_axis(9, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[2][1] + ea.z() * absca[0][1];
        rb = eb.x() * absca[1][2] + eb.z() * absca[1][0];
        s = (t.x() * ca[2][1] - t.z() * ca[0][1]).abs() - (ra + rb);
        let normal = Vec3::new(ca[2][1], 0.0, -ca[0][1]);
        match track_edge_axis(10, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[2][2] + ea.z() * absca[0][2];
        rb = eb.x() * absca[1][1] + eb.y() * absca[1][0];
        s = (t.x() * ca[2][2] - t.z() * ca[0][2]).abs() - (ra + rb);
        let normal = Vec3::new(ca[2][2], 0.0, -ca[0][2]);
        match track_edge_axis(11, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[1][0] + ea.y() * absca[0][0];
        rb = eb.y() * absca[2][2] + eb.z() * absca[2][1];
        s = (t.y() * ca[0][0] - t.x() * ca[1][0]).abs() - (ra + rb);
        let normal = Vec3::new(-ca[1][0], ca[0][0], 0.0);
        match track_edge_axis(12, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[1][1] + ea.y() * absca[0][1];
        rb = eb.x() * absca[2][2] + eb.z() * absca[2][0];
        s = (t.y() * ca[0][1] - t.x() * ca[1][1]).abs() - (ra + rb);
        let normal = Vec3::new(-ca[1][1], ca[0][1], 0.0);
        match track_edge_axis(13, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }

        ra = ea.x() * absca[1][2] + ea.y() * absca[0][2];
        rb = eb.x() * absca[2][1] + eb.y() * absca[2][0];
        s = (t.y() * ca[0][2] - t.x() * ca[1][2]).abs() - (ra + rb);
        let normal = Vec3::new(-ca[1][2], ca[0][2], 0.0);
        match track_edge_axis(14, s, emax, normal) {
            TrackEdgeAxis::None => return None,
            TrackEdgeAxis::Some { max, axis, normal } => {
                emax = max;
                eaxis = axis;
                ne = normal;
            }
            _ => {}
        }
    }

    const REL_TOLERANCE: f32 = 0.95;
    const ABS_TOLERANCE: f32 = 0.01;

    let axis;
    let smax;
    let mut n;
    let facemax = amax.max(bmax);
    if REL_TOLERANCE * emax > facemax + ABS_TOLERANCE {
        axis = eaxis;
        smax = emax;
        n = ne;
    } else if REL_TOLERANCE * bmax > amax + ABS_TOLERANCE {
        axis = baxis;
        smax = bmax;
        n = nb;
    } else {
        axis = aaxis;
        smax = amax;
        n = na;
    }

    if n.dot(btx.translation() - atx.translation()) < 0.0 {
        n = -n;
    }

    if axis == u32::MAX {
        return None;
    }

    if axis < 6 {
        let rtx;
        let itx;
        let er;
        let ei;
        let flip;

        if axis < 3 {
            rtx = atx;
            itx = btx;
            er = ea;
            ei = eb;
            flip = false;
        } else {
            rtx = btx;
            itx = atx;
            er = eb;
            ei = ea;
            flip = true;
            n = -n;
        }

        let incident = compute_incident_face(itx, ei, n);
        let refeb = compute_reference_edges_and_basis(er, rtx, n, axis);
        let clip_edges = refeb.clip_edges;
        let basis = refeb.basis;
        let e = refeb.e;

        let clip = clip(rtx.translation(), e, clip_edges, basis, incident);
        let out = clip.out;

        if out.len() > 0 {
            let normal = if flip { -n } else { n };

            let mut contacts = SmallVec::new();
            for (v, d) in out {
                let contact = Contact {
                    position: v.v,
                    penetration: d,
                };
                contacts.push(contact);
            }
            Some(Manifold {
                body1: a.body,
                body2: b.body,
                normal,
                penetration: smax,
                contacts,
            })
        } else {
            None
        }
    } else {
        let mut n = atx.rotation() * n;

        if n.dot(btx.translation() - atx.translation()) < 0.0 {
            n = -n;
        }

        let [pa, qa] = support_edge(atx, ea, n);
        let [pb, qb] = support_edge(btx, eb, -n);

        let [ca, cb] = edges_contact(pa, qa, pb, qb);

        let normal = n;
        Some(Manifold {
            body1: a.body,
            body2: b.body,
            normal,
            penetration: smax,
            contacts: smallvec![Contact {
                position: (ca + cb) * 0.5,
                penetration: smax,
            }],
        })
    }
}
