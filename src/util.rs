use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeIntersectionResult {
  NoIntersection,
  PointIntersection(Vec2),
  LineIntersection(Vec2, Vec2),
}

// Find the intersection of two line segments. Line segments cannot intersect at
// end points (although if one line's end point is present on the interior of
// the other line, that will be an intersection). The same line segment is also
// considered a line intersection.
pub fn edge_intersection(
  line_1: (Vec2, Vec2),
  line_2: (Vec2, Vec2),
) -> EdgeIntersectionResult {
  // An implementation of Schneider and Eberly line intersection.

  let line_1_vector = line_1.1 - line_1.0;
  let line_2_vector = line_2.1 - line_2.0;

  let relative_start = line_2.0 - line_1.0;
  let cross = line_1_vector.perp_dot(line_2_vector);
  let cross_squared = cross * cross;

  if cross_squared > 0.0 {
    // Line segments are not parallel, so either they intersect at a point or
    // not at all.
    let s = relative_start.perp_dot(line_2_vector) / cross;
    if s < 0.0 || 1.0 < s {
      return EdgeIntersectionResult::NoIntersection;
    }

    let t = relative_start.perp_dot(line_1_vector) / cross;
    if t < 0.0 || 1.0 < t {
      return EdgeIntersectionResult::NoIntersection;
    }

    if (s == 0.0 || s == 1.0) && (t == 0.0 || t == 1.0) {
      return EdgeIntersectionResult::NoIntersection;
    }

    return EdgeIntersectionResult::PointIntersection(
      line_1.0 + s * line_1_vector,
    );
  }
  // Line segments are parallel, so either they are on the same line and
  // overlapping, or there is no intersection.
  let cross = relative_start.perp_dot(line_1_vector);

  if cross.abs() > 0.0 {
    // Lines are not on the same line, so no overlap.
    return EdgeIntersectionResult::NoIntersection;
  }

  let line_1_len_squared = line_1_vector.length_squared();
  let sa = relative_start.dot(line_1_vector) / line_1_len_squared;
  let sb = sa + line_1_vector.dot(line_2_vector) / line_1_len_squared;
  let smin = sa.min(sb);
  let smax = sa.max(sb);

  if smax <= 0.0 || 1.0 <= smin {
    return EdgeIntersectionResult::NoIntersection;
  }

  EdgeIntersectionResult::LineIntersection(
    line_1.0 + smin.max(0.0) * line_1_vector,
    line_1.0 + smax.min(1.0) * line_1_vector,
  )
}

#[cfg(test)]
mod tests {
  use glam::Vec2;

  use crate::util::{edge_intersection, EdgeIntersectionResult};

  #[test]
  fn unaligned_edges_intersect() {
    let line_1 = (Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0));
    let line_2 = (Vec2::new(4.0, 3.0), Vec2::new(4.0, 7.0));
    assert_eq!(
      edge_intersection(line_1, line_2),
      EdgeIntersectionResult::PointIntersection(Vec2::new(4.0, 4.0))
    );
    assert_eq!(
      edge_intersection(line_2, line_1),
      EdgeIntersectionResult::PointIntersection(Vec2::new(4.0, 4.0))
    );
  }

  #[test]
  fn unaligned_edges_dont_intersect() {
    let line_intersects_after_segment_1 =
      (Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0));
    let line_intersects_after_segment_2 =
      (Vec2::new(6.0, 3.0), Vec2::new(6.0, 7.0));
    assert_eq!(
      edge_intersection(
        line_intersects_after_segment_1,
        line_intersects_after_segment_2,
      ),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(
        line_intersects_after_segment_2,
        line_intersects_after_segment_1,
      ),
      EdgeIntersectionResult::NoIntersection
    );

    let line_intersects_before_segment_1 =
      (Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0));
    let line_intersects_before_segment_2 =
      (Vec2::new(1.0, 0.0), Vec2::new(5.0, 0.0));
    assert_eq!(
      edge_intersection(
        line_intersects_before_segment_1,
        line_intersects_before_segment_2,
      ),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(
        line_intersects_before_segment_2,
        line_intersects_before_segment_1,
      ),
      EdgeIntersectionResult::NoIntersection
    );

    let t_intersection_1 = (Vec2::ONE, Vec2::ONE * 5.0);
    let t_intersection_2 = (Vec2::ONE * 3.0, Vec2::new(3.0, 0.0));

    assert_eq!(
      edge_intersection(t_intersection_1, t_intersection_2),
      EdgeIntersectionResult::PointIntersection(t_intersection_2.0)
    );
    assert_eq!(
      edge_intersection(t_intersection_2, t_intersection_1),
      EdgeIntersectionResult::PointIntersection(t_intersection_2.0)
    );
    assert_eq!(
      edge_intersection(
        t_intersection_1,
        (t_intersection_2.1, t_intersection_2.0),
      ),
      EdgeIntersectionResult::PointIntersection(t_intersection_2.0)
    );
    assert_eq!(
      edge_intersection(
        (t_intersection_2.1, t_intersection_2.0),
        t_intersection_1,
      ),
      EdgeIntersectionResult::PointIntersection(t_intersection_2.0)
    );
  }

  #[test]
  fn edges_intersect_at_point() {
    let line_1 = (Vec2::new(-1.0, 2.0), Vec2::new(1.0, 1.0));
    let line_2 = (Vec2::new(1.0, 1.0), Vec2::new(3.0, 1.0));
    let line_3 = (Vec2::new(3.0, 1.0), Vec2::new(7.0, 9.0));
    let t_line = (Vec2::new(2.0, 1.0), Vec2::new(2.0, 3.0));

    assert_eq!(
      edge_intersection(line_1, line_2),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_2, line_1),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_2, line_3),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_3, line_2),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_2, t_line),
      EdgeIntersectionResult::PointIntersection(t_line.0),
    );
    assert_eq!(
      edge_intersection(t_line, line_2),
      EdgeIntersectionResult::PointIntersection(t_line.0),
    );
    assert_eq!(
      edge_intersection(line_2, (t_line.1, t_line.0)),
      EdgeIntersectionResult::PointIntersection(t_line.0),
    );
    assert_eq!(
      edge_intersection((t_line.1, t_line.0), line_2),
      EdgeIntersectionResult::PointIntersection(t_line.0),
    );
  }

  #[test]
  fn aligned_edges_intersect() {
    let start_line = (Vec2::ONE, Vec2::ONE * 4.0);
    let overlapping_line = (Vec2::ONE * 2.0, Vec2::ONE * 7.0);
    let covering_line = (Vec2::ONE * -1.0, Vec2::ONE * 7.0);
    let covered_line = (Vec2::ONE * 2.0, Vec2::ONE * 3.0);
    let offset_line_1 = (start_line.0 + Vec2::Y, start_line.1 + Vec2::Y);
    let offset_line_2 = (Vec2::ONE * 5.0, Vec2::ONE * 7.0);

    assert_eq!(
      edge_intersection(start_line, overlapping_line,),
      EdgeIntersectionResult::LineIntersection(
        Vec2::ONE * 2.0,
        Vec2::ONE * 4.0
      )
    );
    assert_eq!(
      edge_intersection(overlapping_line, start_line,),
      EdgeIntersectionResult::LineIntersection(
        Vec2::ONE * 2.0,
        Vec2::ONE * 4.0
      )
    );

    assert_eq!(
      edge_intersection(start_line, covering_line),
      EdgeIntersectionResult::LineIntersection(start_line.0, start_line.1)
    );
    assert_eq!(
      edge_intersection(covering_line, start_line),
      EdgeIntersectionResult::LineIntersection(start_line.0, start_line.1)
    );

    assert_eq!(
      edge_intersection(start_line, covered_line),
      EdgeIntersectionResult::LineIntersection(covered_line.0, covered_line.1)
    );
    assert_eq!(
      edge_intersection(covered_line, start_line),
      EdgeIntersectionResult::LineIntersection(covered_line.0, covered_line.1)
    );

    assert_eq!(
      edge_intersection(start_line, offset_line_1),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(offset_line_1, start_line),
      EdgeIntersectionResult::NoIntersection
    );

    assert_eq!(
      edge_intersection(start_line, offset_line_2),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(offset_line_2, start_line),
      EdgeIntersectionResult::NoIntersection
    );
  }

  #[test]
  fn aligned_edges_dont_intersect_at_point() {
    let line_1 = (Vec2::ONE, Vec2::ONE * 3.0);
    let line_2 = (Vec2::ONE * 3.0, Vec2::ONE * 7.0);
    let line_3 = (Vec2::ONE * 7.0, Vec2::ONE * 10.0);

    assert_eq!(
      edge_intersection(line_1, line_2),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_2, line_1),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_2, line_3),
      EdgeIntersectionResult::NoIntersection
    );
    assert_eq!(
      edge_intersection(line_3, line_2),
      EdgeIntersectionResult::NoIntersection
    );
  }

  #[test]
  fn edge_intersecting_self() {
    let line = (Vec2::ONE, Vec2::ONE * 5.0);

    // There should be a line intersection if the same line is passed in (even
    // if end points are not intersections).
    assert_eq!(
      edge_intersection(line, line),
      EdgeIntersectionResult::LineIntersection(line.0, line.1)
    );
  }
}
