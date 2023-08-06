use std::{cmp::Reverse, collections::BinaryHeap};

use glam::Vec2;

mod util;

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
  subject: &Polygon,
  clip: &Polygon,
  _operation: Operation,
) -> Polygon {
  let mut event_queue = BinaryHeap::new();
  let mut event_relations = Vec::new();

  create_events_for_polygon(
    subject,
    /* is_subject= */ true,
    &mut event_queue,
    &mut event_relations,
  );
  create_events_for_polygon(
    clip,
    /* is_subject= */ false,
    &mut event_queue,
    &mut event_relations,
  );
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

// The relationship of the event to the rest of the edges. While `Event` is
// immutable, the EventRelation can change over the course of the algorithm.
#[derive(Default, Clone, PartialEq, Debug)]
struct EventRelation {
  // The ID of the point that this edge connects to. This can change through
  // intersections.
  sibling_id: usize,
  // The point that this edge connects to. This can change through
  // intersections.
  sibling_point: Vec2,
  // Indicates if this edge represents an inside-outside transition into the
  // polygon.
  in_out: bool,
  // Same as `in_out`, but for the other polygon (if this event refers to the
  // subject, this will refer to the clip).
  other_in_out: bool,
  // Whether the edge is in the result.
  in_result: bool,
  // The ID of the previous event in the sweep line that was in the result.
  prev_in_result: Option<usize>,
}

// Creates a left and right event for each edge in the polygon.
fn create_events_for_polygon(
  polygon: &Polygon,
  is_subject: bool,
  event_queue: &mut BinaryHeap<Reverse<Event>>,
  event_relations: &mut Vec<EventRelation>,
) {
  for contour in polygon.contours.iter() {
    for point_index in 0..contour.len() {
      let next_point_index =
        if point_index == contour.len() - 1 { 0 } else { point_index + 1 };

      let point_1 = contour[point_index];
      let point_2 = contour[next_point_index];
      let (event_1_left, event_2_left) =
        match lex_order_points(&point_1, &point_2) {
          std::cmp::Ordering::Equal => continue, // Ignore degenerate edges.
          std::cmp::Ordering::Less => (true, false),
          std::cmp::Ordering::Greater => (false, true),
        };

      let event_id_1 = event_relations.len();
      let event_id_2 = event_relations.len() + 1;

      event_queue.push(Reverse(Event {
        event_id: event_id_1,
        point: point_1,
        left: event_1_left,
        is_subject,
        other_point: point_2,
      }));
      event_queue.push(Reverse(Event {
        event_id: event_id_2,
        point: point_2,
        left: event_2_left,
        is_subject,
        other_point: point_1,
      }));

      event_relations.push(EventRelation {
        sibling_id: event_id_2,
        sibling_point: point_2,
        ..Default::default()
      });
      event_relations.push(EventRelation {
        sibling_id: event_id_1,
        sibling_point: point_1,
        ..Default::default()
      });
    }
  }
}

// An event that can be sorted into the sweep line. The sweep line data
// structure will hold the edges currently intersecting the sweep line in
// order from top to bottom. Note the event will always be a left event, since
// right events will remove the associated left event (so the sweep line will
// never contain a right event).
struct SweepLineEvent(Event);

impl PartialEq for SweepLineEvent {
  fn eq(&self, other: &Self) -> bool {
    self.0.event_id == other.0.event_id
  }
}
impl PartialOrd for SweepLineEvent {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    // This is primarily use for a sorted Vec, so here we will say "prefer" to
    // mean less.

    // If the other edge is colinear, order the events normally.
    if self.is_colinear(other) {
      return self.0.partial_cmp(&other.0);
    }

    if self.0.point.x == other.0.point.x {
      return match self.0.point.y.partial_cmp(&other.0.point.y).unwrap() {
        // The left points are equal, so what determines order is the right
        // points, aka the slopes of the edges.
        std::cmp::Ordering::Equal => Some(point_relative_to_line(
          self.0.point,
          self.0.other_point,
          other.0.other_point,
        )),
        // The x coordinate is still the same, so the order of the edges is
        // determined by the vertical position.
        ord => Some(ord),
      };
    }

    // Otherwise, find the event that is leftmost and order based on its line.
    Some(match self.0.cmp(&other.0) {
      // The left points are not equal, so the events cannot be equal.
      std::cmp::Ordering::Equal => unreachable!(),
      std::cmp::Ordering::Greater => {
        point_relative_to_line(other.0.point, other.0.other_point, self.0.point)
          .reverse()
      }
      std::cmp::Ordering::Less => {
        point_relative_to_line(self.0.point, self.0.other_point, other.0.point)
      }
    })
  }
}
impl Eq for SweepLineEvent {}
impl Ord for SweepLineEvent {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).unwrap()
  }
}

impl SweepLineEvent {
  // Returns if the `other` edge is colinear with the `self` edge.
  fn is_colinear(&self, other: &Self) -> bool {
    point_relative_to_line(self.0.point, self.0.other_point, other.0.point)
      .is_eq()
      && point_relative_to_line(
        self.0.point,
        self.0.other_point,
        other.0.other_point,
      )
      .is_eq()
  }
}

#[cfg(test)]
mod tests;
