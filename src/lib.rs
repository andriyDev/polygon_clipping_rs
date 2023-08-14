use std::{
  cmp::Reverse,
  collections::{BinaryHeap, HashMap},
};

use glam::Vec2;
use util::{edge_intersection, EdgeIntersectionResult};

mod util;

#[derive(Clone, PartialEq, Debug)]
pub struct Polygon {
  pub contours: Vec<Vec<Vec2>>,
}

impl Polygon {
  // Computes the bounding box (min, max) of the polygon. Returns None if there
  // are no vertices.
  pub fn compute_bounds(&self) -> Option<(Vec2, Vec2)> {
    self.contours.iter().flatten().fold(None, |bounds, &point| {
      Some(match bounds {
        None => (point, point),
        Some((min, max)) => (min.min(point), max.max(point)),
      })
    })
  }
}

// The source of an edge.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct SourceEdge {
  // Whether the edge is from the subject polygon (otherwise, the clip
  // polygon).
  pub is_from_subject: bool,
  // The index of the contour in the source polygon.
  pub contour: usize,
  // The edge in the contour of the source polygon.
  pub edge: usize,
}

// The result of performing a boolean operation.
#[derive(Clone, PartialEq, Debug)]
pub struct BooleanResult {
  // The resulting polygon.
  pub polygon: Polygon,
  // The source of each edge in `polygon`. `source_edge` will have one entry
  // per contour in `polygon` and each entry will have the same number of edges
  // as that contour in `polygon`.
  pub contour_source_edges: Vec<Vec<SourceEdge>>,
}

pub fn intersection(subject: &Polygon, clip: &Polygon) -> BooleanResult {
  perform_boolean(subject, clip, Operation::Intersection)
}

pub fn union(subject: &Polygon, clip: &Polygon) -> BooleanResult {
  perform_boolean(subject, clip, Operation::Union)
}

pub fn difference(subject: &Polygon, clip: &Polygon) -> BooleanResult {
  perform_boolean(subject, clip, Operation::Difference)
}

pub fn xor(subject: &Polygon, clip: &Polygon) -> BooleanResult {
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
  operation: Operation,
) -> BooleanResult {
  // Turns `polygon` into the corresponding `BooleanResult`.
  fn polygon_to_boolean_result(
    polygon: &Polygon,
    is_subject: bool,
  ) -> BooleanResult {
    BooleanResult {
      polygon: polygon.clone(),
      contour_source_edges: polygon
        .contours
        .iter()
        .enumerate()
        .map(|(contour_index, contour)| {
          (0..contour.len())
            .map(|index| SourceEdge {
              is_from_subject: is_subject,
              contour: contour_index,
              edge: index,
            })
            .collect()
        })
        .collect(),
    }
  }

  // This is just an optimization. If the bounding boxes of each polygon do not
  // intersect, we can trivially compute the boolean operation. This does mean
  // we won't "normalize" the polygons (e.g., removing empty contours), but that
  // is a totally fine tradeoff for the speed.
  let subject_bounds = subject.compute_bounds();
  let clip_bounds = clip.compute_bounds();
  match (subject_bounds, clip_bounds) {
    (None, None) => {
      return BooleanResult {
        polygon: Polygon { contours: vec![] },
        contour_source_edges: vec![],
      }
    }
    (Some(_), None) => {
      return if operation == Operation::Intersection {
        BooleanResult {
          polygon: Polygon { contours: vec![] },
          contour_source_edges: vec![],
        }
      } else {
        polygon_to_boolean_result(subject, /* is_subject= */ true)
      };
    }
    (None, Some(_)) => {
      return if operation == Operation::Intersection
        || operation == Operation::Difference
      {
        BooleanResult {
          polygon: Polygon { contours: vec![] },
          contour_source_edges: vec![],
        }
      } else {
        polygon_to_boolean_result(clip, /* is_subject= */ false)
      };
    }
    (Some((subject_min, subject_max)), Some((clip_min, clip_max))) => {
      if subject_max.x < clip_min.x
        || subject_max.y < clip_min.y
        || clip_max.x < subject_min.x
        || clip_max.y < subject_min.y
      {
        return match operation {
          Operation::Intersection => BooleanResult {
            polygon: Polygon { contours: vec![] },
            contour_source_edges: vec![],
          },
          Operation::Difference => {
            polygon_to_boolean_result(subject, /* is_subject= */ true)
          }
          Operation::Union | Operation::XOR => {
            let mut subject_result =
              polygon_to_boolean_result(subject, /* is_subject= */ true);
            let mut clip_result =
              polygon_to_boolean_result(clip, /* is_subject= */ false);
            subject_result
              .polygon
              .contours
              .append(&mut clip_result.polygon.contours);
            subject_result
              .contour_source_edges
              .append(&mut clip_result.contour_source_edges);
            subject_result
          }
        };
      }
    }
  }

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

  let result_events =
    subdivide_edges(event_queue, &mut event_relations, operation);
  join_contours(result_events, event_relations, operation)
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

  // Determine whether `self` and `relation` imply the edge is in the result
  // based on the operation.
  fn in_result(&self, relation: &EventRelation, operation: Operation) -> bool {
    if relation.edge_coincidence_type != EdgeCoincidenceType::NoCoincidence {
      return relation.edge_coincidence_type.in_result(operation);
    }
    match operation {
      // The edge is in the result iff it is inside the other polygon, aka if
      // the closest edge below is an out-in transition.
      Operation::Intersection => !relation.other_in_out,
      // The edge is in the result iff it is outside the other polygon, aka if
      // the closest edge below is an in-out transition (or is non-existent).
      Operation::Union => relation.other_in_out,
      // The edge is in the result iff it is either the subject and we are
      // outside the clip polygon (the closest edge below is an in-out
      // transition), or it is the clip and we are inside the clip polygon (the
      // closest edge below is an out-in transition).
      Operation::Difference => self.is_subject == relation.other_in_out,
      // Every edge is part of the result.
      Operation::XOR => true,
    }
  }

  // Determines whether `self` and `relation` that are in the result is an
  // in-out transition or not.
  fn result_in_out(
    &self,
    relation: &EventRelation,
    operation: Operation,
  ) -> bool {
    // These variables make the logic below more intuitive (e.g. a && b for
    // intersection).
    let is_inside_self = !relation.in_out;
    let is_inside_other = !relation.other_in_out;

    // For all of these cases, we already know the edge is in the result.
    let result_out_in = match operation {
      // The edge is an out-in transition iff we are inside both this polygon
      // and the other polygon.
      Operation::Intersection => is_inside_self && is_inside_other,
      // The edge is an out-in transition iff we are inside either this polygon
      // or the other polygon.
      Operation::Union => is_inside_self || is_inside_other,
      // The edge is an out-in transition iff we are in the subject polygon and
      // not in the clip polygon (nothing was subtracted), or we are in the clip
      // polygon and not in the subject (since we know the edge is in the
      // result, we must have just left the subject polygon).
      Operation::Difference => {
        if self.is_subject {
          is_inside_self && !is_inside_other
        } else {
          !is_inside_self && is_inside_other
        }
      }
      // The edge is an out-in transition iff the polygons are in opposite
      // states.
      Operation::XOR => is_inside_self != is_inside_other,
    };

    !result_out_in
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
  // The type of coincidence between another edge.
  edge_coincidence_type: EdgeCoincidenceType,
  // The edge that this event comes from. This can change for coincident edges
  // to prefer to report the subject edge.
  source_edge: SourceEdge,
}

// The type of edge coincidence (overlapping edges).
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeCoincidenceType {
  // A standard edge before any coincident edge has been detected.
  #[default]
  NoCoincidence,
  // There is a coincident edge and it has the same in-out transition as this
  // edge.
  SameTransition,
  // There is a coincient edge and it has a different in-out transition as
  // this edge.
  DifferentTransition,
  // There is a coincident edge, but we only need one of these edges in the
  // result - this edge will not be in the result.
  DuplicateCoincidence,
}

impl EdgeCoincidenceType {
  fn in_result(&self, operation: Operation) -> bool {
    match self {
      EdgeCoincidenceType::NoCoincidence => panic!(),
      EdgeCoincidenceType::DuplicateCoincidence => false,
      EdgeCoincidenceType::SameTransition => {
        operation == Operation::Intersection || operation == Operation::Union
      }
      EdgeCoincidenceType::DifferentTransition => {
        operation == Operation::Difference
      }
    }
  }
}

// Creates a left and right event for each edge in the polygon. Returns the
// bounds of the polygon for convenience.
fn create_events_for_polygon(
  polygon: &Polygon,
  is_subject: bool,
  event_queue: &mut BinaryHeap<Reverse<Event>>,
  event_relations: &mut Vec<EventRelation>,
) {
  for (contour_index, contour) in polygon.contours.iter().enumerate() {
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
        source_edge: SourceEdge {
          is_from_subject: is_subject,
          contour: contour_index,
          edge: point_index,
        },
        ..Default::default()
      });
      event_relations.push(EventRelation {
        sibling_id: event_id_1,
        sibling_point: point_1,
        source_edge: SourceEdge {
          is_from_subject: is_subject,
          contour: contour_index,
          edge: point_index,
        },
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

// Check for intersections between two events in the sweep line. `new_event` is
// the event just inserted into the sweep line and `existing_event` is the event
// that was already in the sweep line.
fn check_for_intersection(
  new_event: &Event,
  existing_event: &Event,
  event_queue: &mut BinaryHeap<Reverse<Event>>,
  event_relations: &mut Vec<EventRelation>,
  operation: Operation,
) {
  match edge_intersection(
    (new_event.point, event_relations[new_event.event_id].sibling_point),
    (
      existing_event.point,
      event_relations[existing_event.event_id].sibling_point,
    ),
  ) {
    EdgeIntersectionResult::NoIntersection => {} // Do nothing.
    EdgeIntersectionResult::PointIntersection(point) => {
      // Split the edges, but only if the the split point isn't at an end point.
      if point != new_event.point
        && point != event_relations[new_event.event_id].sibling_point
      {
        split_edge(new_event, point, event_queue, event_relations);
      }
      if point != existing_event.point
        && point != event_relations[existing_event.event_id].sibling_point
      {
        split_edge(existing_event, point, event_queue, event_relations);
      }
    }
    EdgeIntersectionResult::LineIntersection(start, end) => {
      let new_event_coincident_event_id;
      match (
        start == new_event.point,
        end == event_relations[new_event.event_id].sibling_point,
      ) {
        (true, true) => {
          // The edge is fully covered, so no new splits are necessary.
          new_event_coincident_event_id = new_event.event_id;
        }
        (false, false) => {
          split_edge(new_event, end, event_queue, event_relations);
          new_event_coincident_event_id =
            split_edge(new_event, start, event_queue, event_relations);
        }
        (true, false) => {
          split_edge(new_event, end, event_queue, event_relations);
          new_event_coincident_event_id = new_event.event_id;
        }
        (false, true) => {
          new_event_coincident_event_id =
            split_edge(new_event, start, event_queue, event_relations);
        }
      }

      let existing_event_coincident_event_id;
      match (
        start == existing_event.point,
        end == event_relations[existing_event.event_id].sibling_point,
      ) {
        (true, true) => {
          // The edge is fully covered, so no new splits are necessary.
          existing_event_coincident_event_id = existing_event.event_id;
        }
        (false, false) => {
          split_edge(existing_event, end, event_queue, event_relations);
          existing_event_coincident_event_id =
            split_edge(existing_event, start, event_queue, event_relations);
        }
        (true, false) => {
          split_edge(existing_event, end, event_queue, event_relations);
          existing_event_coincident_event_id = existing_event.event_id;
        }
        (false, true) => {
          existing_event_coincident_event_id =
            split_edge(existing_event, start, event_queue, event_relations);
        }
      }

      let same_transition = event_relations[new_event.event_id].in_out
        == event_relations[existing_event.event_id].in_out;

      // The prev_in_result of the new edge can sometimes equal the pre-existing
      // edge. Since the edges are intersecting, their prev_in_result
      // should match (since neither is "more important").
      event_relations[new_event_coincident_event_id].prev_in_result =
        event_relations[existing_event.event_id].prev_in_result;

      // We say the "primary" coincident edge is the one that will represent
      // both edges. The "duplicate" coincident edge will not contribute to the
      // final polygon.
      let (primary_edge_event_id, duplicate_edge_event_id) =
        if event_relations[existing_event_coincident_event_id].in_result {
          (existing_event_coincident_event_id, new_event_coincident_event_id)
        } else {
          (new_event_coincident_event_id, existing_event_coincident_event_id)
        };
      // In the final result, we want to prefer subject edges over clip edges,
      // so change the primary edge (which is the only one possibly in the
      // result) to use the subject edge if one of them is a clip edge.
      match (
        event_relations[new_event.event_id].source_edge.is_from_subject,
        event_relations[existing_event.event_id].source_edge.is_from_subject,
      ) {
        // Neither edge is "preferred", so just go with the defaults.
        (true, true) => {}
        (false, false) => {}
        // The subject edge should be preferred, so assign those.
        (true, false) => {
          let source_edge = event_relations[new_event.event_id].source_edge;
          event_relations[primary_edge_event_id].source_edge = source_edge;
          let sibling_id = event_relations[primary_edge_event_id].sibling_id;
          event_relations[sibling_id].source_edge = source_edge;
        }
        (false, true) => {
          let source_edge =
            event_relations[existing_event.event_id].source_edge;
          event_relations[primary_edge_event_id].source_edge = source_edge;
          let sibling_id = event_relations[primary_edge_event_id].sibling_id;
          event_relations[sibling_id].source_edge = source_edge;
        }
      }

      let primary_edge_relation = &mut event_relations[primary_edge_event_id];
      primary_edge_relation.edge_coincidence_type = if same_transition {
        EdgeCoincidenceType::SameTransition
      } else {
        EdgeCoincidenceType::DifferentTransition
      };
      primary_edge_relation.in_result =
        primary_edge_relation.edge_coincidence_type.in_result(operation);

      let duplicate_edge_relation =
        &mut event_relations[duplicate_edge_event_id];
      duplicate_edge_relation.edge_coincidence_type =
        EdgeCoincidenceType::DuplicateCoincidence;
      duplicate_edge_relation.in_result = false;
    }
  }
}

// Splits an edge into two parts at `point`. Siblings are updated for the
// existing events and new events are generated. Returns the index of the left
// event of the new edge.
fn split_edge(
  edge_event: &Event,
  point: Vec2,
  event_queue: &mut BinaryHeap<Reverse<Event>>,
  event_relations: &mut Vec<EventRelation>,
) -> usize {
  let (sibling_id, sibling_point, source_edge) = {
    let relation = &event_relations[edge_event.event_id];
    (relation.sibling_id, relation.sibling_point, relation.source_edge)
  };

  let split_1_id = event_relations.len();
  let split_2_id = event_relations.len() + 1;

  event_queue.push(Reverse(Event {
    event_id: split_1_id,
    point,
    left: false,
    is_subject: edge_event.is_subject,
    other_point: edge_event.point,
  }));
  event_queue.push(Reverse(Event {
    event_id: split_2_id,
    point,
    left: true,
    is_subject: edge_event.is_subject,
    other_point: edge_event.other_point,
  }));

  event_relations.push(EventRelation {
    sibling_id: edge_event.event_id,
    sibling_point: edge_event.point,
    source_edge,
    ..Default::default()
  });
  event_relations.push(EventRelation {
    sibling_id,
    sibling_point,
    source_edge,
    ..Default::default()
  });

  let edge_event_relation = &mut event_relations[edge_event.event_id];
  edge_event_relation.sibling_id = split_1_id;
  edge_event_relation.sibling_point = point;

  let edge_sibling_relation = &mut event_relations[sibling_id];
  edge_sibling_relation.sibling_id = split_2_id;
  edge_sibling_relation.sibling_point = point;

  split_2_id
}

// Determines the flags in `event_relation`. These are used to determine whether
// edges are in the result or not. This assumes `prev_event` is already in the
// sweep line and has had its own flags computed.
fn set_information(
  (event, event_relation): (&Event, &mut EventRelation),
  prev_event: Option<(&Event, &EventRelation)>,
  operation: Operation,
) {
  match prev_event {
    None => {
      // There is no previous event, so this must be the external contour of
      // one of the polygons.
      event_relation.in_out = false;
      // Even if there is no previous event, we mark it as an in-out
      // transition since this treats the other as being "outside".
      event_relation.other_in_out = true;
    }
    Some((prev_event, prev_event_relation)) => {
      if event.is_subject == prev_event.is_subject {
        // The events are from the same polygon, so this event should be the
        // opposite of `prev_event`.
        event_relation.in_out = !prev_event_relation.in_out;
        // The nearest other polygon's edge stays the same.
        event_relation.other_in_out = prev_event_relation.other_in_out;
      } else {
        // `prev_event` is from the other polygon, so the nearest edge of its
        // other polygon is the same as this event. Flip its sign just as
        // above.
        event_relation.in_out = !prev_event_relation.other_in_out;
        event_relation.other_in_out = if !prev_event.is_vertical() {
          // When the previous edge is not vertical, since `prev_event` is the
          // other polygon, we just copy the in_out directly.
          prev_event_relation.in_out
        } else {
          // When the previous edge is vertical, this edge really cares about
          // the in_out transition of the top of the previous edge. For
          // horizontal edges, this is the same as in_out, but for vertical
          // edges, the top of the edge has the opposite in_out.
          !prev_event_relation.in_out
        };
      }

      // The in_result part is obvious. If the previous event is vertical, we do
      // not want to use it as prev_in_result, since we are just skimming the
      // edge of the polygon.
      event_relation.prev_in_result =
        if prev_event_relation.in_result && !prev_event.is_vertical() {
          Some(prev_event.event_id)
        } else {
          prev_event_relation.prev_in_result
        };
    }
  }

  event_relation.in_result = event.in_result(event_relation, operation);
}

// Goes through the `event_queue` and subdivides intersecting edges. Returns a
// Vec of events corresponding to the edges that are in the final result based
// on `operation`.
fn subdivide_edges(
  mut event_queue: BinaryHeap<Reverse<Event>>,
  event_relations: &mut Vec<EventRelation>,
  operation: Operation,
) -> Vec<Event> {
  let mut sweep_line = Vec::new();
  let mut result = Vec::new();
  while let Some(Reverse(event)) = event_queue.pop() {
    if event.left {
      let sweep_line_event = SweepLineEvent(event.clone());
      let pos = sweep_line
        .binary_search(&sweep_line_event)
        .expect_err("event is new and must be inserted");
      sweep_line.insert(pos, sweep_line_event);
      if pos == 0 {
        set_information(
          (&event, &mut event_relations[event.event_id]),
          /* prev_event= */ None,
          operation,
        )
      } else {
        let prev_event = &sweep_line[pos - 1].0;
        {
          let (event_relation, prev_event_relation) = borrow_two_mut(
            event_relations,
            event.event_id,
            prev_event.event_id,
          );
          set_information(
            (&event, event_relation),
            Some((prev_event, prev_event_relation)),
            operation,
          );
        }
        // TODO: See if it matters that we reordered prev and next checks.
        check_for_intersection(
          &event,
          prev_event,
          &mut event_queue,
          event_relations,
          operation,
        );
      }
      if pos + 1 < sweep_line.len() {
        // If the inserted event isn't last, check for intersection with next
        // event.
        let next_event = &sweep_line[pos + 1].0;
        check_for_intersection(
          &event,
          next_event,
          &mut event_queue,
          event_relations,
          operation,
        );
      }
    } else {
      // The right edge event is in the result if its left edge event is also in
      // the result.
      event_relations[event.event_id].in_result =
        event_relations[event_relations[event.event_id].sibling_id].in_result;
      let pos = sweep_line
        .binary_search(&order_sibling(&event, &event_relations[event.event_id]))
        .expect("this is a right event, so the left event must have already been inserted.");
      sweep_line.remove(pos);
      if 0 < pos && pos < sweep_line.len() {
        let (prev_event, next_event) =
          (&sweep_line[pos - 1].0, &sweep_line[pos].0);
        check_for_intersection(
          prev_event,
          next_event,
          &mut event_queue,
          event_relations,
          operation,
        );
      }
    }

    if event_relations[event.event_id].in_result {
      result.push(event);
    }
  }

  // Only keep events that are still in the result by the end. With no
  // coincident edges, in_result can never change after the first pass. However,
  // with coincident edges, an edge that previously thought it was in the result
  // may no longer be in the result. Consider the subject and the clip being the
  // same, and the operation being difference or XOR. The first subject edge
  // will have no previous event in the sweep line, so it will think it is in
  // the result. Then the clip edge will be processed and now the edge is no
  // longer in the result.
  result.retain(|event| event_relations[event.event_id].in_result);

  result
}

// Borrows two elements from a slice mutably. It should be unreachable to ever
// call this with two of the same index.
fn borrow_two_mut<T>(slice: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
  if a < b {
    let (left, right) = slice.split_at_mut(b);
    (&mut left[a], &mut right[0])
  } else if b < a {
    let (left, right) = slice.split_at_mut(a);
    (&mut right[0], &mut left[b])
  } else {
    unreachable!();
  }
}

// Derives a SweepLineEvent corresponding to the sibling of `event`. `event` is
// assumed to be a right event (since that is the only time you need to
// determine the order sibling).
fn order_sibling(
  event: &Event,
  event_relation: &EventRelation,
) -> SweepLineEvent {
  SweepLineEvent(Event {
    event_id: event_relation.sibling_id,
    point: event_relation.sibling_point,
    left: true,
    is_subject: event.is_subject,
    other_point: event.point,
  })
}

// The flags for each event used to derive contours.
#[derive(Default)]
struct EventContourFlags {
  // The index into the result events that this event corresponds to.
  result_id: usize,
  // Whether this event corresponds to an in-out transition of the output
  // polygon.
  result_in_out: bool,
  // The ID of the contour this event belongs to.
  contour_id: usize,
  // The ID of the parent contour if it exists.
  parent_id: Option<usize>,
  // Whether this event has already been processed - events do not need to be
  // processed multipled times.
  processed: bool,
  // The depth of the contour. Even if a shell, odd if a hole.
  depth: u32,
}

// Computes the depth and the ID of the parent contour (if the parent exists).
fn compute_depth(
  event: &Event,
  event_relations: &[EventRelation],
  event_id_to_contour_flags: &HashMap<usize, EventContourFlags>,
) -> (u32, Option<usize>) {
  match event_relations[event.event_id].prev_in_result {
    None => (0, None),
    Some(prev_in_result) => {
      let prev_contour_flags = &event_id_to_contour_flags[&prev_in_result];

      if !prev_contour_flags.result_in_out {
        (prev_contour_flags.depth + 1, Some(prev_contour_flags.contour_id))
      } else {
        (prev_contour_flags.depth, prev_contour_flags.parent_id)
      }
    }
  }
}

// Computes the contour starting at `start_event`. Events that are part of the
// contour will be assigned the `depth`, `contour_id`, and `parent_contour_id`.
fn compute_contour(
  start_event: &Event,
  contour_id: usize,
  depth: u32,
  parent_contour_id: Option<usize>,
  event_relations: &[EventRelation],
  event_id_to_contour_flags: &mut HashMap<usize, EventContourFlags>,
  result_events: &[Event],
) -> (Vec<Vec2>, Vec<SourceEdge>) {
  let mut contour = Vec::new();
  let mut contour_source_edges = Vec::new();
  contour.push(start_event.point);
  contour_source_edges.push(event_relations[start_event.event_id].source_edge);
  let mut current_event = event_to_sibling_and_mark(
    start_event,
    contour_id,
    depth,
    parent_contour_id,
    event_relations,
    event_id_to_contour_flags,
    &result_events,
  );

  while current_event.point != start_event.point {
    let result_id =
      event_id_to_contour_flags[&current_event.event_id].result_id;
    if 0 < result_id
      && result_events[result_id - 1].point == current_event.point
    {
      current_event = &result_events[result_id - 1];
      event_id_to_contour_flags
        .get_mut(&current_event.event_id)
        .unwrap()
        .processed = true;
    } else {
      // One of the adjacent events in `result_events` must be connected to
      // the current event, panic otherwise.
      debug_assert!(result_id + 1 < result_events.len());
      debug_assert_eq!(
        result_events[result_id + 1].point,
        current_event.point,
        "result_id={}, event_id={}",
        result_id,
        current_event.event_id,
      );
      current_event = &result_events[result_id + 1];
      event_id_to_contour_flags
        .get_mut(&current_event.event_id)
        .unwrap()
        .processed = true;
    }
    contour.push(current_event.point);
    contour_source_edges
      .push(event_relations[current_event.event_id].source_edge);
    current_event = event_to_sibling_and_mark(
      current_event,
      contour_id,
      depth,
      parent_contour_id,
      &event_relations,
      event_id_to_contour_flags,
      &result_events,
    );
  }

  (contour, contour_source_edges)
}

// Finds the sibling of `event`, sets its flags to match the provided arguments,
// and returns the sibling event.
fn event_to_sibling_and_mark<'a>(
  event: &Event,
  contour_id: usize,
  depth: u32,
  parent_contour_id: Option<usize>,
  event_relations: &[EventRelation],
  event_id_to_contour_flags: &mut HashMap<usize, EventContourFlags>,
  result_events: &'a [Event],
) -> &'a Event {
  let sibling_id = event_relations[event.event_id].sibling_id;
  let contour_relation =
    event_id_to_contour_flags.get_mut(&sibling_id).unwrap();
  contour_relation.processed = true;
  contour_relation.contour_id = contour_id;
  contour_relation.depth = depth;
  contour_relation.parent_id = parent_contour_id;
  &result_events[contour_relation.result_id]
}

// Determines the contours of the result polygon from the `result_events`.
fn join_contours(
  result_events: Vec<Event>,
  event_relations: Vec<EventRelation>,
  operation: Operation,
) -> BooleanResult {
  let mut event_id_to_contour_flags = result_events
    .iter()
    .enumerate()
    .map(|(result_id, event)| {
      let event_meta = &event_relations[event.event_id];
      (
        event.event_id,
        EventContourFlags {
          result_id,
          result_in_out: event.result_in_out(event_meta, operation),
          ..Default::default()
        },
      )
    })
    .collect::<HashMap<_, _>>();

  let mut contours = Vec::new();
  let mut contour_source_edges = Vec::new();
  for result_event in result_events.iter() {
    if event_id_to_contour_flags[&result_event.event_id].processed {
      continue;
    }
    let (depth, parent_contour_id) =
      compute_depth(result_event, &event_relations, &event_id_to_contour_flags);
    let (mut contour, mut source_edges_for_contour) = compute_contour(
      result_event,
      contours.len(),
      depth,
      parent_contour_id,
      &event_relations,
      &mut event_id_to_contour_flags,
      &result_events,
    );

    if depth % 2 == 1 {
      contour.reverse();
      source_edges_for_contour.reverse();
    }

    contours.push(contour);
    contour_source_edges.push(source_edges_for_contour);
  }

  BooleanResult { polygon: Polygon { contours }, contour_source_edges }
}

#[cfg(test)]
mod tests;
