use std::{
  cmp::Reverse,
  collections::BinaryHeap,
  f32::{EPSILON, INFINITY},
};

use glam::Vec2;
use rand::seq::SliceRandom;

use crate::{
  check_for_intersection, create_events_for_polygon, difference, intersection,
  split_edge, union, xor, BooleanResult, EdgeCoincidenceType, Event,
  EventRelation, Operation, Polygon, SourceEdge,
};

#[test]
fn split_edge_events_ordered_correctly() {
  let expected_events = [
    // Edge start events.
    Event {
      event_id: 100,
      point: Vec2::new(3.0, 2.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(5.0, 2.0),
    },
    Event {
      event_id: 90,
      point: Vec2::new(3.5, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(5.0, 3.0),
    },
    // Edge intersection events.
    Event {
      event_id: 95,
      point: Vec2::new(4.25, 2.0),
      left: false,
      is_subject: true,
      other_point: Vec2::new(3.5, 1.0),
    },
    Event {
      event_id: 93,
      point: Vec2::new(4.25, 2.0),
      left: false,
      is_subject: false,
      other_point: Vec2::new(3.0, 2.0),
    },
    Event {
      event_id: 105,
      point: Vec2::new(4.25, 2.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(5.0, 2.0),
    },
    Event {
      event_id: 101,
      point: Vec2::new(4.25, 2.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(5.0, 3.0),
    },
    // Edge end events.
    Event {
      event_id: 97,
      point: Vec2::new(5.0, 2.0),
      left: false,
      is_subject: false,
      other_point: Vec2::new(3.0, 2.0),
    },
    Event {
      event_id: 89,
      point: Vec2::new(5.0, 3.0),
      left: false,
      is_subject: true,
      other_point: Vec2::new(3.5, 1.0),
    },
  ];

  assert!(
    expected_events[0] < expected_events[1],
    "left={:?} right={:?}",
    expected_events[0],
    expected_events[1]
  );
  assert!(
    expected_events[1] < expected_events[2],
    "left={:?} right={:?}",
    expected_events[1],
    expected_events[2]
  );
  assert!(
    expected_events[2] < expected_events[3],
    "left={:?} right={:?}",
    expected_events[2],
    expected_events[3]
  );
  assert!(
    expected_events[3] < expected_events[4],
    "left={:?} right={:?}",
    expected_events[3],
    expected_events[4]
  );
  assert!(
    expected_events[4] < expected_events[5],
    "left={:?} right={:?}",
    expected_events[4],
    expected_events[5]
  );
  assert!(
    expected_events[5] < expected_events[6],
    "left={:?} right={:?}",
    expected_events[5],
    expected_events[6]
  );
  assert!(
    expected_events[6] < expected_events[7],
    "left={:?} right={:?}",
    expected_events[6],
    expected_events[7]
  );

  let mut sorted_events = expected_events.iter().cloned().collect::<Vec<_>>();
  sorted_events.shuffle(&mut rand::thread_rng());
  sorted_events.sort();

  assert_eq!(sorted_events, expected_events);
}

// Consumes the `event_queue` and turns it into a sorted Vec of events.
fn event_queue_to_vec(event_queue: BinaryHeap<Reverse<Event>>) -> Vec<Event> {
  let mut event_queue = event_queue
    .into_sorted_vec()
    .iter()
    .map(|e| e.0.clone())
    .collect::<Vec<_>>();
  // into_sorted_vec returns the sort of Reverse(Event), so reverse the order to
  // get the sort order of Event.
  event_queue.reverse();

  event_queue
}

#[test]
fn no_bounds_for_empty_polygon() {
  assert_eq!(
    Polygon { contours: vec![vec![], vec![], vec![]] }.compute_bounds(),
    None
  );
}

#[test]
fn computes_bounds_for_non_empty_polygon() {
  assert_eq!(
    Polygon {
      contours: vec![
        vec![Vec2::new(1.0, 1.0), Vec2::new(5.0, 2.0)],
        vec![],
        vec![Vec2::new(2.0, 5.0), Vec2::new(3.0, 3.0)]
      ]
    }
    .compute_bounds(),
    Some((Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0)))
  );
}

#[test]
fn creates_events_for_polygon() {
  let polygon = Polygon {
    contours: vec![
      vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(3.0, 3.0),
        Vec2::new(1.0, 3.0),
      ],
      vec![
        Vec2::new(4.0, 1.0),
        Vec2::new(5.0, 1.0),
        Vec2::new(6.0, 2.0),
        Vec2::new(5.0, 2.0),
      ],
    ],
  };

  let mut event_queue = BinaryHeap::new();
  let mut event_relations = Vec::new();
  create_events_for_polygon(
    &polygon,
    /* is_subject= */ true,
    &mut event_queue,
    &mut event_relations,
    /* x_limit= */ INFINITY,
  );
  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 0,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 1.0),
      },
      Event {
        event_id: 7,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(1.0, 3.0),
      },
      Event {
        event_id: 6,
        point: Vec2::new(1.0, 3.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      },
      Event {
        event_id: 5,
        point: Vec2::new(1.0, 3.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 3.0),
      },
      Event {
        event_id: 1,
        point: Vec2::new(3.0, 1.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      },
      Event {
        event_id: 2,
        point: Vec2::new(3.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 3.0),
      },
      Event {
        event_id: 4,
        point: Vec2::new(3.0, 3.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 3.0),
      },
      Event {
        event_id: 3,
        point: Vec2::new(3.0, 3.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(3.0, 1.0),
      },
      Event {
        event_id: 8,
        point: Vec2::new(4.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(5.0, 1.0),
      },
      Event {
        event_id: 15,
        point: Vec2::new(4.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(5.0, 2.0),
      },
      Event {
        event_id: 9,
        point: Vec2::new(5.0, 1.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(4.0, 1.0),
      },
      Event {
        event_id: 10,
        point: Vec2::new(5.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(6.0, 2.0),
      },
      Event {
        event_id: 14,
        point: Vec2::new(5.0, 2.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(4.0, 1.0),
      },
      Event {
        event_id: 13,
        point: Vec2::new(5.0, 2.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(6.0, 2.0),
      },
      Event {
        event_id: 11,
        point: Vec2::new(6.0, 2.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(5.0, 1.0),
      },
      Event {
        event_id: 12,
        point: Vec2::new(6.0, 2.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(5.0, 2.0),
      },
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(3.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(3.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: Vec2::new(3.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 5,
        sibling_point: Vec2::new(1.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(3.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 7,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 6,
        sibling_point: Vec2::new(1.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 9,
        sibling_point: Vec2::new(5.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 8,
        sibling_point: Vec2::new(4.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 11,
        sibling_point: Vec2::new(6.0, 2.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 1 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 10,
        sibling_point: Vec2::new(5.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 1 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 13,
        sibling_point: Vec2::new(5.0, 2.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 12,
        sibling_point: Vec2::new(6.0, 2.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 15,
        sibling_point: Vec2::new(4.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 3 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 14,
        sibling_point: Vec2::new(5.0, 2.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 1, edge: 3 },
        ..Default::default()
      },
    ]
  );
}

#[test]
fn creates_events_for_polygon_with_x_limit() {
  let polygon = Polygon {
    contours: vec![
      vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(3.0, 3.0),
        Vec2::new(1.0, 3.0),
      ],
      vec![
        Vec2::new(4.0, 1.0),
        Vec2::new(5.0, 1.0),
        Vec2::new(6.0, 2.0),
        Vec2::new(5.0, 2.0),
      ],
    ],
  };

  let mut event_queue = BinaryHeap::new();
  let mut event_relations = Vec::new();
  create_events_for_polygon(
    &polygon,
    /* is_subject= */ true,
    &mut event_queue,
    &mut event_relations,
    /* x_limit= */ 2.0,
  );
  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 0,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 1.0),
      },
      Event {
        event_id: 5,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(1.0, 3.0),
      },
      Event {
        event_id: 4,
        point: Vec2::new(1.0, 3.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      },
      Event {
        event_id: 3,
        point: Vec2::new(1.0, 3.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 3.0),
      },
      Event {
        event_id: 1,
        point: Vec2::new(3.0, 1.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      },
      Event {
        event_id: 2,
        point: Vec2::new(3.0, 3.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 3.0),
      },
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(3.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(1.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: Vec2::new(3.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 5,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(1.0, 3.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ..Default::default()
      },
    ]
  );
}

#[test]
fn splits_edges() {
  let mut event_queue = BinaryHeap::new();
  let mut event_relations = vec![
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(0.0, 0.0),
      source_edge: SourceEdge { is_from_subject: true, contour: 4, edge: 20 },
      ..Default::default()
    },
  ];

  const SPLIT_EDGE: Vec2 = Vec2::new(0.75, 0.75);
  assert_eq!(
    split_edge(
      &Event {
        event_id: 1,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(0.0, 0.0),
      },
      SPLIT_EDGE,
      &mut event_queue,
      &mut event_relations,
    ),
    3
  );

  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 2,
        point: SPLIT_EDGE,
        left: false,
        is_subject: true,
        other_point: Vec2::new(0.0, 0.0),
      },
      Event {
        event_id: 3,
        point: SPLIT_EDGE,
        left: true,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      }
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 3,
        sibling_point: SPLIT_EDGE,
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: SPLIT_EDGE,
        source_edge: SourceEdge { is_from_subject: true, contour: 4, edge: 20 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 4, edge: 20 },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(0.0, 0.0),
        source_edge: SourceEdge { is_from_subject: true, contour: 4, edge: 20 },
        ..Default::default()
      },
    ]
  );
}

#[test]
fn check_for_intersection_finds_no_intersection() {
  let mut event_queue = BinaryHeap::new();
  let mut event_relations = vec![
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(3.0, 4.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(1.0, 2.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 3,
      sibling_point: Vec2::new(3.0, 3.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 2,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
  ];
  let expected_event_relations = event_relations.clone();

  check_for_intersection(
    &Event {
      event_id: 0,
      point: Vec2::new(1.0, 2.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(3.0, 4.0),
    },
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(3.0, 3.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Union,
  );

  // No new events.
  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(event_queue, []);
  assert_eq!(event_relations, expected_event_relations);
}

#[test]
fn check_for_intersection_finds_point_intersection() {
  let mut event_queue = BinaryHeap::new();
  let mut event_relations = vec![
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(3.0, 3.0),
      source_edge: SourceEdge { is_from_subject: false, contour: 4, edge: 20 },
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(1.0, 2.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 3,
      sibling_point: Vec2::new(3.0, 4.0),
      source_edge: SourceEdge { is_from_subject: true, contour: 13, edge: 37 },
      ..Default::default()
    },
    EventRelation {
      sibling_id: 2,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
  ];

  check_for_intersection(
    &Event {
      event_id: 0,
      point: Vec2::new(1.0, 2.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(3.0, 3.0),
    },
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(3.0, 4.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Union,
  );

  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 6,
        point: Vec2::new(2.0, 2.5),
        left: false,
        is_subject: false,
        other_point: Vec2::new(1.0, 2.0),
      },
      Event {
        event_id: 4,
        point: Vec2::new(2.0, 2.5),
        left: false,
        is_subject: false,
        other_point: Vec2::new(1.0, 2.0),
      },
      Event {
        event_id: 5,
        point: Vec2::new(2.0, 2.5),
        left: true,
        is_subject: false,
        other_point: Vec2::new(3.0, 3.0),
      },
      Event {
        event_id: 7,
        point: Vec2::new(2.0, 2.5),
        left: true,
        is_subject: false,
        other_point: Vec2::new(3.0, 3.0),
      },
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(2.0, 2.5),
        source_edge: SourceEdge {
          is_from_subject: false,
          contour: 4,
          edge: 20
        },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 5,
        sibling_point: Vec2::new(2.0, 2.5),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 6,
        sibling_point: Vec2::new(2.0, 2.5),
        source_edge: SourceEdge {
          is_from_subject: true,
          contour: 13,
          edge: 37
        },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 7,
        sibling_point: Vec2::new(2.0, 2.5),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(1.0, 2.0),
        source_edge: SourceEdge {
          is_from_subject: false,
          contour: 4,
          edge: 20
        },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(3.0, 3.0),
        source_edge: SourceEdge {
          is_from_subject: false,
          contour: 4,
          edge: 20
        },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: Vec2::new(1.0, 1.0),
        source_edge: SourceEdge {
          is_from_subject: true,
          contour: 13,
          edge: 37
        },
        ..Default::default()
      },
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(3.0, 4.0),
        source_edge: SourceEdge {
          is_from_subject: true,
          contour: 13,
          edge: 37
        },
        ..Default::default()
      },
    ]
  );
}

#[test]
fn check_for_intersection_finds_fully_overlapped_line() {
  let mut event_queue = BinaryHeap::new();
  let original_event_relations = vec![
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(3.0, 3.0),
      in_out: true,
      prev_in_result: Some(1337),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(0.0, 0.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 3,
      sibling_point: Vec2::new(2.0, 2.0),
      in_out: false,
      prev_in_result: Some(420),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 2,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
  ];

  let mut event_relations = original_event_relations.clone();
  check_for_intersection(
    &Event {
      event_id: 0,
      point: Vec2::new(0.0, 0.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(3.0, 3.0),
    },
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(2.0, 2.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Union,
  );

  let event_queue = event_queue_to_vec(event_queue);
  let expected_event_queue = [
    Event {
      event_id: 6,
      point: Vec2::new(1.0, 1.0),
      left: false,
      is_subject: false,
      other_point: Vec2::new(0.0, 0.0),
    },
    Event {
      event_id: 7,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(2.0, 2.0),
    },
    Event {
      event_id: 4,
      point: Vec2::new(2.0, 2.0),
      left: false,
      is_subject: false,
      other_point: Vec2::new(0.0, 0.0),
    },
    Event {
      event_id: 5,
      point: Vec2::new(2.0, 2.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(3.0, 3.0),
    },
  ];
  assert_eq!(event_queue, expected_event_queue);
  let expected_event_relations = [
    EventRelation {
      sibling_id: 6,
      sibling_point: Vec2::new(1.0, 1.0),
      in_out: true,
      prev_in_result: Some(1337),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 5,
      sibling_point: Vec2::new(2.0, 2.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 3,
      sibling_point: Vec2::new(2.0, 2.0),
      prev_in_result: Some(420),
      // Event 2 is the existing event, but is not in the result, so this is a
      // duplicate.
      edge_coincidence_type: EdgeCoincidenceType::DuplicateCoincidence,
      ..Default::default()
    },
    EventRelation {
      sibling_id: 2,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 7,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(3.0, 3.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(0.0, 0.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 4,
      sibling_point: Vec2::new(2.0, 2.0),
      // The event should copy the prev_in_result from event 2.
      prev_in_result: Some(420),
      // This event comes from event 0 which is the new event, so this will be
      // the "primary" coincident edge.
      edge_coincidence_type: EdgeCoincidenceType::DifferentTransition,
      ..Default::default()
    },
  ];
  assert_eq!(event_relations, expected_event_relations);

  let mut event_queue = BinaryHeap::new();
  event_relations = original_event_relations.clone();
  check_for_intersection(
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(2.0, 2.0),
    },
    &Event {
      event_id: 0,
      point: Vec2::new(0.0, 0.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(3.0, 3.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Union,
  );

  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(event_queue, expected_event_queue);
  assert_eq!(
    event_relations,
    [
      expected_event_relations[0].clone(),
      expected_event_relations[1].clone(),
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(2.0, 2.0),
        // `prev_in_result` was copied from event 0, since that is the existing
        // event.
        prev_in_result: Some(1337),
        // Event 2 is the new event (and the existing event is not in the
        // result), so it will be the "primary" coincident edge.
        edge_coincidence_type: EdgeCoincidenceType::DifferentTransition,
        ..Default::default()
      },
      expected_event_relations[3].clone(),
      expected_event_relations[4].clone(),
      expected_event_relations[5].clone(),
      expected_event_relations[6].clone(),
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(2.0, 2.0),
        prev_in_result: None,
        edge_coincidence_type: EdgeCoincidenceType::DuplicateCoincidence,
        ..Default::default()
      },
    ]
  );
}

#[test]
fn check_for_intersection_finds_partially_overlapped_lines() {
  let mut event_queue = BinaryHeap::new();
  let original_event_relations = vec![
    EventRelation {
      sibling_id: 1,
      sibling_point: Vec2::new(2.0, 2.0),
      in_out: false,
      prev_in_result: Some(1337),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 0,
      sibling_point: Vec2::new(0.0, 0.0),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 3,
      sibling_point: Vec2::new(3.0, 3.0),
      in_out: false,
      in_result: true,
      prev_in_result: Some(420),
      ..Default::default()
    },
    EventRelation {
      sibling_id: 2,
      sibling_point: Vec2::new(1.0, 1.0),
      ..Default::default()
    },
  ];

  let mut event_relations = original_event_relations.clone();
  check_for_intersection(
    &Event {
      event_id: 0,
      point: Vec2::new(0.0, 0.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(2.0, 2.0),
    },
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(3.0, 3.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Intersection,
  );

  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 4,
        point: Vec2::new(1.0, 1.0),
        left: false,
        is_subject: false,
        other_point: Vec2::new(0.0, 0.0),
      },
      Event {
        event_id: 5,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: false,
        other_point: Vec2::new(3.0, 3.0),
      },
      Event {
        event_id: 6,
        point: Vec2::new(2.0, 2.0),
        left: false,
        is_subject: false,
        other_point: Vec2::new(0.0, 0.0),
      },
      Event {
        event_id: 7,
        point: Vec2::new(2.0, 2.0),
        left: true,
        is_subject: false,
        other_point: Vec2::new(3.0, 3.0),
      },
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(1.0, 1.0),
        prev_in_result: Some(1337),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 5,
        sibling_point: Vec2::new(1.0, 1.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 6,
        sibling_point: Vec2::new(2.0, 2.0),
        prev_in_result: Some(420),
        // The operation is intersection, so two edges means the "primary" edge
        // is in the result.
        in_result: true,
        edge_coincidence_type: EdgeCoincidenceType::SameTransition,
        ..Default::default()
      },
      EventRelation {
        sibling_id: 7,
        sibling_point: Vec2::new(2.0, 2.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(0.0, 0.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(2.0, 2.0),
        prev_in_result: Some(420),
        edge_coincidence_type: EdgeCoincidenceType::DuplicateCoincidence,
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: Vec2::new(1.0, 1.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(3.0, 3.0),
        ..Default::default()
      },
    ]
  );

  let mut event_queue = BinaryHeap::new();
  event_relations = original_event_relations.clone();
  check_for_intersection(
    &Event {
      event_id: 2,
      point: Vec2::new(1.0, 1.0),
      left: true,
      is_subject: true,
      other_point: Vec2::new(3.0, 3.0),
    },
    &Event {
      event_id: 0,
      point: Vec2::new(0.0, 0.0),
      left: true,
      is_subject: false,
      other_point: Vec2::new(2.0, 2.0),
    },
    &mut event_queue,
    &mut event_relations,
    Operation::Difference,
  );

  let event_queue = event_queue_to_vec(event_queue);
  assert_eq!(
    event_queue,
    [
      Event {
        event_id: 6,
        point: Vec2::new(1.0, 1.0),
        left: false,
        is_subject: false,
        other_point: Vec2::new(0.0, 0.0),
      },
      Event {
        event_id: 7,
        point: Vec2::new(1.0, 1.0),
        left: true,
        is_subject: false,
        other_point: Vec2::new(2.0, 2.0),
      },
      Event {
        event_id: 4,
        point: Vec2::new(2.0, 2.0),
        left: false,
        is_subject: true,
        other_point: Vec2::new(1.0, 1.0),
      },
      Event {
        event_id: 5,
        point: Vec2::new(2.0, 2.0),
        left: true,
        is_subject: true,
        other_point: Vec2::new(3.0, 3.0),
      },
    ]
  );
  assert_eq!(
    event_relations,
    [
      EventRelation {
        sibling_id: 6,
        sibling_point: Vec2::new(1.0, 1.0),
        prev_in_result: Some(1337),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 7,
        sibling_point: Vec2::new(1.0, 1.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 4,
        sibling_point: Vec2::new(2.0, 2.0),
        // `prev_in_result` was copied from event 0.
        prev_in_result: Some(1337),
        // Event 0 is not in the result, so this event is chosen as the
        // "primary" coincident event.
        edge_coincidence_type: EdgeCoincidenceType::SameTransition,
        // Cleared because the operation is difference.
        in_result: false,
        ..Default::default()
      },
      EventRelation {
        sibling_id: 5,
        sibling_point: Vec2::new(2.0, 2.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 2,
        sibling_point: Vec2::new(1.0, 1.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 3,
        sibling_point: Vec2::new(3.0, 3.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 0,
        sibling_point: Vec2::new(0.0, 0.0),
        ..Default::default()
      },
      EventRelation {
        sibling_id: 1,
        sibling_point: Vec2::new(2.0, 2.0),
        edge_coincidence_type: EdgeCoincidenceType::DuplicateCoincidence,
        ..Default::default()
      },
    ]
  );
}

#[test]
fn boolean_of_rhombuses() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(3.5, 1.0),
      Vec2::new(5.0, 3.0),
      Vec2::new(3.0, 3.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(3.0, 2.0),
      Vec2::new(5.0, 2.0),
      Vec2::new(7.0, 4.0),
      Vec2::new(5.0, 4.0),
    ]],
  };

  assert_eq!(
    union(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(1.0, 1.0),
          Vec2::new(3.5, 1.0),
          Vec2::new(4.25, 2.0),
          Vec2::new(5.0, 2.0),
          Vec2::new(7.0, 4.0),
          Vec2::new(5.0, 4.0),
          Vec2::new(4.0, 3.0),
          Vec2::new(3.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    intersection(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(3.0, 2.0),
          Vec2::new(4.25, 2.0),
          Vec2::new(5.0, 3.0),
          Vec2::new(4.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    difference(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(1.0, 1.0),
          Vec2::new(3.5, 1.0),
          Vec2::new(4.25, 2.0),
          Vec2::new(3.0, 2.0),
          Vec2::new(4.0, 3.0),
          Vec2::new(3.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    xor(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![
          vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(3.5, 1.0),
            Vec2::new(4.25, 2.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(4.0, 3.0),
            Vec2::new(3.0, 3.0),
          ],
          vec![
            Vec2::new(4.0, 3.0),
            Vec2::new(5.0, 3.0),
            Vec2::new(4.25, 2.0),
            Vec2::new(5.0, 2.0),
            Vec2::new(7.0, 4.0),
            Vec2::new(5.0, 4.0),
          ]
        ]
      },
      contour_source_edges: vec![
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ],
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        ]
      ],
    }
  );
}

#[test]
fn boolean_of_squares() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(3.0, 3.0),
      Vec2::new(1.0, 3.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, 2.0),
      Vec2::new(4.0, 2.0),
      Vec2::new(4.0, 4.0),
      Vec2::new(2.0, 4.0),
    ]],
  };

  assert_eq!(
    union(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(1.0, 1.0),
          Vec2::new(3.0, 1.0),
          Vec2::new(3.0, 2.0),
          Vec2::new(4.0, 2.0),
          Vec2::new(4.0, 4.0),
          Vec2::new(2.0, 4.0),
          Vec2::new(2.0, 3.0),
          Vec2::new(1.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    intersection(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(2.0, 2.0),
          Vec2::new(3.0, 2.0),
          Vec2::new(3.0, 3.0),
          Vec2::new(2.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    difference(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![vec![
          Vec2::new(1.0, 1.0),
          Vec2::new(3.0, 1.0),
          Vec2::new(3.0, 2.0),
          Vec2::new(2.0, 2.0),
          Vec2::new(2.0, 3.0),
          Vec2::new(1.0, 3.0),
        ]]
      },
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );

  assert_eq!(
    xor(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![
          vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(2.0, 3.0),
            Vec2::new(1.0, 3.0),
          ],
          vec![
            Vec2::new(2.0, 3.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(2.0, 4.0),
          ]
        ]
      },
      contour_source_edges: vec![
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ],
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        ]
      ],
    }
  );
}

#[test]
fn add_and_remove_squares() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(3.0, 3.0),
      Vec2::new(1.0, 3.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(2.0, 2.0),
      Vec2::new(1.0, 2.0),
    ]],
  };

  // All boolean operations between the clip and the subject.
  assert_eq!(
    intersection(&subject, &clip),
    BooleanResult {
      polygon: clip.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );
  let expected_union = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(3.0, 3.0),
      Vec2::new(1.0, 3.0),
      Vec2::new(1.0, 2.0),
    ]],
  };
  assert_eq!(
    union(&subject, &clip),
    BooleanResult {
      polygon: expected_union.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]],
    }
  );

  let expected_difference = BooleanResult {
    polygon: Polygon {
      contours: vec![vec![
        Vec2::new(1.0, 2.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(3.0, 3.0),
        Vec2::new(1.0, 3.0),
      ]],
    },
    contour_source_edges: vec![vec![
      SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
    ]],
  };
  assert_eq!(difference(&subject, &clip), expected_difference);

  let xor_result = xor(&subject, &clip);
  assert_eq!(xor_result, expected_difference);

  assert_eq!(
    intersection(&xor_result.polygon, &clip),
    BooleanResult {
      polygon: Polygon { contours: vec![] },
      contour_source_edges: vec![]
    }
  );
  assert_eq!(
    union(&xor_result.polygon, &clip),
    BooleanResult {
      polygon: expected_union,
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 5 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      ]],
    }
  );
}

#[test]
fn cut_and_fill_hole() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(5.0, 1.0),
      Vec2::new(5.0, 5.0),
      Vec2::new(1.0, 5.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, 2.0),
      Vec2::new(4.0, 2.0),
      Vec2::new(4.0, 4.0),
      Vec2::new(2.0, 4.0),
    ]],
  };

  let expected_subject_with_hole = Polygon {
    contours: vec![
      vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(5.0, 1.0),
        Vec2::new(5.0, 5.0),
        Vec2::new(1.0, 5.0),
      ],
      vec![
        Vec2::new(2.0, 4.0),
        Vec2::new(4.0, 4.0),
        Vec2::new(4.0, 2.0),
        Vec2::new(2.0, 2.0),
      ],
    ],
  };

  let expected_difference = BooleanResult {
    polygon: expected_subject_with_hole.clone(),
    contour_source_edges: vec![
      vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ],
      vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
      ],
    ],
  };
  assert_eq!(difference(&subject, &clip), expected_difference);
  assert_eq!(xor(&subject, &clip), expected_difference);

  assert_eq!(
    union(&expected_subject_with_hole, &clip),
    BooleanResult {
      polygon: subject.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]]
    }
  );
  assert_eq!(
    xor(&expected_subject_with_hole, &clip),
    BooleanResult {
      polygon: subject.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]]
    }
  );

  assert_eq!(
    union(&expected_subject_with_hole, &subject),
    BooleanResult {
      polygon: subject.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ]]
    }
  );
  assert_eq!(
    xor(&expected_subject_with_hole, &subject),
    BooleanResult {
      polygon: clip,
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 1, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 1, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 1, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 1, edge: 3 },
      ]]
    }
  );
}

#[test]
fn partially_overlapping_edges_are_split() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(2.5, 1.0),
      Vec2::new(4.0, 1.0),
      Vec2::new(4.0, 4.0),
      Vec2::new(3.9, 4.0),
      Vec2::new(1.1, 4.0),
      Vec2::new(1.0, 4.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(4.0, 2.0),
      Vec2::new(4.0, 3.0),
      Vec2::new(3.0, 4.0),
      Vec2::new(2.0, 4.0),
      Vec2::new(1.0, 3.0),
      Vec2::new(1.0, 2.0),
    ]],
  };

  let subdivided_subject = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 1.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(2.5, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(4.0, 1.0),
      Vec2::new(4.0, 2.0),
      Vec2::new(4.0, 3.0),
      Vec2::new(4.0, 4.0),
      Vec2::new(3.9, 4.0),
      Vec2::new(3.0, 4.0),
      Vec2::new(2.0, 4.0),
      Vec2::new(1.1, 4.0),
      Vec2::new(1.0, 4.0),
      Vec2::new(1.0, 3.0),
      Vec2::new(1.0, 2.0),
    ]],
  };
  let subdivided_clip = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 2.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(2.5, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(4.0, 2.0),
      Vec2::new(4.0, 3.0),
      Vec2::new(3.0, 4.0),
      Vec2::new(2.0, 4.0),
      Vec2::new(1.0, 3.0),
    ]],
  };

  assert_eq!(
    intersection(&subject, &clip),
    BooleanResult {
      polygon: subdivided_clip.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 7 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 5 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
      ]],
    }
  );
  assert_eq!(
    intersection(&clip, &subject),
    BooleanResult {
      polygon: subdivided_clip,
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 7 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 5 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
      ]],
    }
  );
  assert_eq!(
    union(&subject, &clip),
    BooleanResult {
      polygon: subdivided_subject.clone(),
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 5 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
      ]],
    }
  );
  assert_eq!(
    union(&clip, &subject),
    BooleanResult {
      polygon: subdivided_subject,
      contour_source_edges: vec![vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 4 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 5 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 6 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 6 },
      ]],
    }
  );
  assert_eq!(
    difference(&subject, &clip),
    BooleanResult {
      polygon: Polygon {
        contours: vec![
          vec![Vec2::new(1.0, 1.0), Vec2::new(2.0, 1.0), Vec2::new(1.0, 2.0)],
          vec![
            Vec2::new(1.0, 3.0),
            Vec2::new(2.0, 4.0),
            Vec2::new(1.1, 4.0),
            Vec2::new(1.0, 4.0),
          ],
          vec![Vec2::new(3.0, 1.0), Vec2::new(4.0, 1.0), Vec2::new(4.0, 2.0)],
          vec![
            Vec2::new(3.0, 4.0),
            Vec2::new(4.0, 3.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(3.9, 4.0),
          ],
        ]
      },
      contour_source_edges: vec![
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 7 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
        ],
        vec![
          SourceEdge { is_from_subject: false, contour: 0, edge: 5 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 5 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 6 },
        ],
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        ],
        vec![
          SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 4 },
        ]
      ],
    }
  );
  assert_eq!(
    difference(&clip, &subject),
    BooleanResult {
      polygon: Polygon { contours: vec![] },
      contour_source_edges: vec![],
    }
  );
}

#[test]
fn trivially_computes_operations_for_disjoint_bounding_boxes() {
  let subject = Polygon {
    contours: vec![
      vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 2.0),
      ],
      // Empty contour to check that the original polygon is used "verbatim".
      vec![],
      vec![
        Vec2::new(2.5, 2.5),
        Vec2::new(3.5, 2.5),
        Vec2::new(3.5, 3.5),
        Vec2::new(2.5, 3.5),
      ],
    ],
  };

  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(-2.0, 1.0),
      Vec2::new(-1.0, 1.0),
      Vec2::new(-1.0, 2.0),
      Vec2::new(-2.0, 2.0),
    ]],
  };

  let expected_union = BooleanResult {
    polygon: Polygon {
      contours: vec![
        // Subject contours.
        vec![
          Vec2::new(1.0, 1.0),
          Vec2::new(2.0, 1.0),
          Vec2::new(2.0, 2.0),
          Vec2::new(1.0, 2.0),
        ],
        vec![],
        vec![
          Vec2::new(2.5, 2.5),
          Vec2::new(3.5, 2.5),
          Vec2::new(3.5, 3.5),
          Vec2::new(2.5, 3.5),
        ],
        // Clip contours.
        vec![
          Vec2::new(-2.0, 1.0),
          Vec2::new(-1.0, 1.0),
          Vec2::new(-1.0, 2.0),
          Vec2::new(-2.0, 2.0),
        ],
      ],
    },
    contour_source_edges: vec![
      vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ],
      vec![],
      vec![
        SourceEdge { is_from_subject: true, contour: 2, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 3 },
      ],
      vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      ],
    ],
  };
  assert_eq!(union(&subject, &clip), expected_union);
  assert_eq!(xor(&subject, &clip), expected_union);

  assert_eq!(
    intersection(&subject, &clip),
    BooleanResult {
      polygon: Polygon { contours: vec![] },
      contour_source_edges: vec![],
    }
  );
  assert_eq!(
    difference(&subject, &clip),
    BooleanResult {
      polygon: subject.clone(),
      contour_source_edges: vec![
        vec![
          SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
        ],
        vec![],
        vec![
          SourceEdge { is_from_subject: true, contour: 2, edge: 0 },
          SourceEdge { is_from_subject: true, contour: 2, edge: 1 },
          SourceEdge { is_from_subject: true, contour: 2, edge: 2 },
          SourceEdge { is_from_subject: true, contour: 2, edge: 3 },
        ],
      ],
    }
  );
}

#[test]
fn trivially_computes_operations_for_empty_polygons() {
  let non_empty_polygon = Polygon {
    contours: vec![
      vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 2.0),
      ],
      // Empty contour to check that the original polygon is used "verbatim".
      vec![],
      vec![
        Vec2::new(2.5, 2.5),
        Vec2::new(3.5, 2.5),
        Vec2::new(3.5, 3.5),
        Vec2::new(2.5, 3.5),
      ],
    ],
  };

  let empty_polygon = Polygon {
    contours: vec![
      // Empty contour to ensure this doesn't count.
      vec![],
    ],
  };

  let empty_boolean_result = BooleanResult {
    polygon: Polygon { contours: vec![] },
    contour_source_edges: vec![],
  };
  let non_empty_boolean_result_as_subject = BooleanResult {
    polygon: non_empty_polygon.clone(),
    contour_source_edges: vec![
      vec![
        SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      ],
      vec![],
      vec![
        SourceEdge { is_from_subject: true, contour: 2, edge: 0 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 1 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 2 },
        SourceEdge { is_from_subject: true, contour: 2, edge: 3 },
      ],
    ],
  };
  let non_empty_boolean_result_as_clip = BooleanResult {
    polygon: non_empty_polygon.clone(),
    contour_source_edges: vec![
      vec![
        SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      ],
      vec![],
      vec![
        SourceEdge { is_from_subject: false, contour: 2, edge: 0 },
        SourceEdge { is_from_subject: false, contour: 2, edge: 1 },
        SourceEdge { is_from_subject: false, contour: 2, edge: 2 },
        SourceEdge { is_from_subject: false, contour: 2, edge: 3 },
      ],
    ],
  };
  assert_eq!(
    union(&non_empty_polygon, &empty_polygon),
    non_empty_boolean_result_as_subject
  );
  assert_eq!(
    union(&empty_polygon, &non_empty_polygon),
    non_empty_boolean_result_as_clip
  );

  assert_eq!(
    intersection(&non_empty_polygon, &empty_polygon),
    empty_boolean_result
  );
  assert_eq!(
    intersection(&empty_polygon, &non_empty_polygon),
    empty_boolean_result
  );

  assert_eq!(
    difference(&non_empty_polygon, &empty_polygon),
    non_empty_boolean_result_as_subject
  );
  assert_eq!(
    difference(&empty_polygon, &non_empty_polygon),
    empty_boolean_result
  );

  assert_eq!(
    xor(&non_empty_polygon, &empty_polygon),
    non_empty_boolean_result_as_subject
  );
  assert_eq!(
    xor(&empty_polygon, &non_empty_polygon),
    non_empty_boolean_result_as_clip
  );

  assert_eq!(union(&empty_polygon, &empty_polygon), empty_boolean_result);
  assert_eq!(
    intersection(&empty_polygon, &empty_polygon),
    empty_boolean_result
  );
  assert_eq!(difference(&empty_polygon, &empty_polygon), empty_boolean_result);
  assert_eq!(xor(&empty_polygon, &empty_polygon), empty_boolean_result);
}

#[test]
fn floating_point_inaccuracy_polygons() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, 0.0),
      Vec2::new(1.0, 0.0),
      Vec2::new(1.0, -2.0),
      Vec2::new(2.0, -1.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, -0.01),
      Vec2::new(2.0, 0.01),
      Vec2::new(1.0, 0.01),
      Vec2::new(1.0, -0.01),
    ]],
  };

  let BooleanResult { polygon, contour_source_edges } = union(&subject, &clip);
  assert_eq!(
    polygon,
    Polygon {
      contours: vec![vec![
        Vec2::new(1.0, -2.0),
        Vec2::new(2.0, -1.0),
        Vec2::new(2.0, -0.01),
        Vec2::new(2.0, 0.0),
        Vec2::new(2.0, 0.01),
        Vec2::new(1.0, 0.01),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, -0.01),
      ]]
    }
  );
  assert_eq!(
    contour_source_edges,
    vec![vec![
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
    ]]
  );
}

#[test]
fn sweep_line_point_on_other_edge() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(-1.0 + EPSILON, 0.0),
      Vec2::new(-1.0 + EPSILON, 1.0 - EPSILON),
      Vec2::new(-2.0 + EPSILON, 2.0 - EPSILON),
      Vec2::new(-2.0 + EPSILON, 0.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(-2.0 + EPSILON, 0.01 + EPSILON),
      Vec2::new(-2.0 + EPSILON, -0.01 + EPSILON),
      Vec2::new(-1.0, -0.01 + EPSILON),
      Vec2::new(-1.0, 0.01 + EPSILON),
    ]],
  };
  let BooleanResult { polygon, contour_source_edges } = union(&subject, &clip);
  assert_eq!(
    polygon,
    Polygon {
      contours: vec![vec![
        clip.contours[0][1],
        clip.contours[0][2],
        subject.contours[0][0],
        subject.contours[0][1],
        subject.contours[0][2],
        clip.contours[0][0],
        subject.contours[0][3],
      ]]
    }
  );
  assert_eq!(
    contour_source_edges,
    vec![vec![
      SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
    ]]
  );
}

#[test]
fn overlapping_edges_with_extra_on_both_ends() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(2.0, 2.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(4.0, 1.0),
      Vec2::new(3.0, 2.0),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(2.01, 2.0),
      Vec2::new(1.99, 2.0),
      Vec2::new(1.99, 1.0),
      Vec2::new(2.01, 1.0),
    ]],
  };

  let BooleanResult { polygon, contour_source_edges } = union(&subject, &clip);
  assert_eq!(
    polygon,
    Polygon {
      contours: vec![vec![
        Vec2::new(1.99, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.01, 1.0),
        Vec2::new(4.0, 1.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(2.01, 2.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.99, 2.0),
      ]]
    }
  );
  assert_eq!(
    contour_source_edges,
    vec![vec![
      SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
    ]]
  );

  let BooleanResult { polygon, contour_source_edges } =
    intersection(&subject, &clip);
  assert_eq!(
    polygon,
    Polygon {
      contours: vec![vec![
        Vec2::new(2.0, 1.0),
        Vec2::new(2.01, 1.0),
        Vec2::new(2.01, 2.0),
        Vec2::new(2.0, 2.0),
      ]]
    }
  );
  assert_eq!(
    contour_source_edges,
    vec![vec![
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 0 },
    ]]
  );
}

#[test]
fn overlapping_edges_with_extra_on_both_ends_with_epsilon() {
  let subject = Polygon {
    contours: vec![vec![
      Vec2::new(0.0, 1.0 - EPSILON),
      Vec2::new(0.0, 2.0 - EPSILON),
      Vec2::new(-2.0 + EPSILON, 2.0 - EPSILON),
      Vec2::new(-1.0 + EPSILON, 1.0 - EPSILON),
    ]],
  };
  let clip = Polygon {
    contours: vec![vec![
      Vec2::new(-0.01 + EPSILON, 1.0),
      Vec2::new(0.01 - EPSILON, 1.0),
      Vec2::new(0.01 - EPSILON, 2.0 - EPSILON),
      Vec2::new(-0.01 + EPSILON, 2.0 - EPSILON),
    ]],
  };

  let BooleanResult { polygon, contour_source_edges } = union(&subject, &clip);
  assert_eq!(
    polygon,
    Polygon {
      contours: vec![vec![
        Vec2::new(-2.0 + EPSILON, 2.0 - EPSILON),
        Vec2::new(-1.0 + EPSILON, 1.0 - EPSILON),
        Vec2::new(-0.01 + EPSILON, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.01 - EPSILON, 1.0),
        Vec2::new(0.01 - EPSILON, 2.0 - EPSILON),
        Vec2::new(0.0, 2.0 - EPSILON),
        Vec2::new(-0.01 + EPSILON, 2.0 - EPSILON),
      ]]
    }
  );
  assert_eq!(
    contour_source_edges,
    vec![vec![
      SourceEdge { is_from_subject: true, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 3 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 0 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: false, contour: 0, edge: 2 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
      SourceEdge { is_from_subject: true, contour: 0, edge: 1 },
    ]]
  );
}
