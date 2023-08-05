use glam::Vec2;

#[derive(Clone, PartialEq, Debug)]
pub struct Polygon {
  pub contours: Vec<Vec<Vec2>>,
}

pub fn intersection(subject: &Polygon, clip: &Polygon) -> Polygon {
  perform_boolean(subject, clip, Operation::Intersection)
}

pub fn union(subject: &Polygon, clip: &Polygon) -> Polygon {
  perform_boolean(subject, clip, Operation::Union)
}

pub fn difference(subject: &Polygon, clip: &Polygon) -> Polygon {
  perform_boolean(subject, clip, Operation::Difference)
}

pub fn xor(subject: &Polygon, clip: &Polygon) -> Polygon {
  perform_boolean(subject, clip, Operation::XOR)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
  Intersection,
  Union,
  XOR,
  Difference,
}

fn perform_boolean(
  _subject: &Polygon,
  _clip: &Polygon,
  _operation: Operation,
) -> Polygon {
  todo!()
}

// An "event" of an edge. Each edge of a polygon is comprised of a "left" event
// and a "right" event.
#[derive(Clone, Debug)]
struct Event {
  // The id of the event.
  event_id: usize,
  // The point where the event occurs.
  point: Vec2,
  // True iff this is the "left" event of the edge. Left generally refers to
  // the point with the lower x coordinate, although for vertical edges, the
  // left is the point with the lower y coordinate.
  left: bool,
  // Did this event come from the subject or the clip?
  is_subject: bool,
  // The other point of this edge. This point will never change after creation.
  // It is just provided to determine the line that the edge sits on (which
  // also can never change).
  other_point: Vec2,
}

impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    self.event_id == other.event_id
  }
}

impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    // This is primarily used for a min heap, so here we will say "prefer" to
    // mean less.

    // The first thing that matters is the order of points.
    match lex_order_points(&self.point, &other.point) {
      std::cmp::Ordering::Equal => {}
      ord => return Some(ord),
    }
    // Prefer right events to left events. This way the sweep line will contain
    // fewer edges (and be more accurate) as right events remove edges from the
    // sweep line.
    match self.left.cmp(&other.left) {
      std::cmp::Ordering::Equal => {}
      ord => return Some(ord),
    }
    // Prefer horizontal edges to vertical edges. Edges use the previous edge in
    // the sweep line to determine whether they are in the result. If we don't
    // prefer horizontal edges, it is possible for a T intersection to handle
    // the left edge, then the vertical edge, then the right edge (meaning the
    // vertical edge will have nothing in the sweep line to compare against).
    match self.is_vertical().cmp(&other.is_vertical()) {
      std::cmp::Ordering::Equal => {}
      ord => return Some(ord),
    }
    // We know the events share the same point. Prefer the line which slopes
    // above the other one.
    match point_relative_to_line(
      self.point,
      self.other_point,
      other.other_point,
    ) {
      std::cmp::Ordering::Equal => {}
      // If this is a right point, then the point and other_point are in the
      // wrong order, so reverse the ordering.
      ord => return if self.left { Some(ord) } else { Some(ord.reverse()) },
    }
    // Prefer subject edges over clip edges.
    match self.is_subject.cmp(&other.is_subject) {
      std::cmp::Ordering::Equal => {}
      ord => return Some(ord.reverse()),
    }

    Some(self.event_id.cmp(&other.event_id))
  }
}

impl Eq for Event {}

impl Ord for Event {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).unwrap()
  }
}

// Determine the lexical order of `a` and `b`. In other words, sort by x, then
// y.
fn lex_order_points(a: &Vec2, b: &Vec2) -> std::cmp::Ordering {
  match a.x.partial_cmp(&b.x) {
    Some(std::cmp::Ordering::Equal) => {}
    Some(ord) => return ord,
    None => panic!(),
  }
  match a.y.partial_cmp(&b.y) {
    Some(std::cmp::Ordering::Equal) => {}
    Some(ord) => return ord,
    None => panic!(),
  }
  std::cmp::Ordering::Equal
}

impl Event {
  // Determines whether the edge is a vertical edge.
  fn is_vertical(&self) -> bool {
    self.point.x == self.other_point.x
  }
}

// Returns whether `point` is above (Greater) or below (Less) the line defined
// by `a` and `b`. Note if b is to the left of a, the returned ordering will be
// reversed.
fn point_relative_to_line(a: Vec2, b: Vec2, point: Vec2) -> std::cmp::Ordering {
  0.0.partial_cmp(&(b - a).perp_dot(point - a)).unwrap()
}

#[cfg(test)]
mod tests;
