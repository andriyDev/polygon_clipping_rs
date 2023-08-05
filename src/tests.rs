use glam::Vec2;
use rand::seq::SliceRandom;

use crate::{difference, intersection, union, xor, Event, Polygon};

#[test]
fn split_edge_events_ordered_correctly() {
  let expected_events = [
    // Edge start events.
    Event {
      event_id: 100,
      point: Vec2::new(3.0, 2.0),
      is_subject: false,
      left: true,
      other_point: Vec2::new(5.0, 2.0),
    },
    Event {
      event_id: 90,
      point: Vec2::new(3.5, 1.0),
      is_subject: true,
      left: true,
      other_point: Vec2::new(5.0, 3.0),
    },
    // Edge intersection events.
    Event {
      event_id: 95,
      point: Vec2::new(4.25, 2.0),
      is_subject: true,
      left: false,
      other_point: Vec2::new(3.5, 1.0),
    },
    Event {
      event_id: 93,
      point: Vec2::new(4.25, 2.0),
      is_subject: false,
      left: false,
      other_point: Vec2::new(3.0, 2.0),
    },
    Event {
      event_id: 105,
      point: Vec2::new(4.25, 2.0),
      is_subject: false,
      left: true,
      other_point: Vec2::new(5.0, 2.0),
    },
    Event {
      event_id: 101,
      point: Vec2::new(4.25, 2.0),
      is_subject: true,
      left: true,
      other_point: Vec2::new(5.0, 3.0),
    },
    // Edge end events.
    Event {
      event_id: 97,
      point: Vec2::new(5.0, 2.0),
      is_subject: false,
      left: false,
      other_point: Vec2::new(3.0, 2.0),
    },
    Event {
      event_id: 89,
      point: Vec2::new(5.0, 3.0),
      is_subject: true,
      left: false,
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

  dbg!(&sorted_events);
  dbg!(&expected_events);
  assert_eq!(sorted_events, expected_events);
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
    Polygon {
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
    }
  );

  assert_eq!(
    intersection(&subject, &clip),
    Polygon {
      contours: vec![vec![
        Vec2::new(3.0, 2.0),
        Vec2::new(4.25, 2.0),
        Vec2::new(5.0, 3.0),
        Vec2::new(4.0, 3.0),
      ]]
    }
  );

  assert_eq!(
    difference(&subject, &clip),
    Polygon {
      contours: vec![vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(3.5, 1.0),
        Vec2::new(4.25, 2.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(4.0, 3.0),
        Vec2::new(3.0, 3.0),
      ]]
    }
  );

  assert_eq!(
    xor(&subject, &clip),
    Polygon {
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
    Polygon {
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
    }
  );

  assert_eq!(
    intersection(&subject, &clip),
    Polygon {
      contours: vec![vec![
        Vec2::new(2.0, 2.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(3.0, 3.0),
        Vec2::new(2.0, 3.0),
      ]]
    }
  );

  assert_eq!(
    difference(&subject, &clip),
    Polygon {
      contours: vec![vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(2.0, 3.0),
        Vec2::new(1.0, 3.0),
      ]]
    }
  );

  assert_eq!(
    xor(&subject, &clip),
    Polygon {
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
  assert_eq!(intersection(&subject, &clip), clip);
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
  assert_eq!(union(&subject, &clip), expected_union);

  let expected_difference = Polygon {
    contours: vec![vec![
      Vec2::new(1.0, 2.0),
      Vec2::new(2.0, 2.0),
      Vec2::new(2.0, 1.0),
      Vec2::new(3.0, 1.0),
      Vec2::new(3.0, 3.0),
      Vec2::new(1.0, 3.0),
    ]],
  };
  assert_eq!(difference(&subject, &clip), expected_difference);

  let xor_result = xor(&subject, &clip);
  assert_eq!(xor_result, expected_difference);

  dbg!(&xor_result);
  assert_eq!(intersection(&xor_result, &clip), Polygon { contours: vec![] });
  assert_eq!(union(&xor_result, &clip), expected_union);
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

  assert_eq!(difference(&subject, &clip), expected_subject_with_hole);
  assert_eq!(xor(&subject, &clip), expected_subject_with_hole);

  assert_eq!(union(&expected_subject_with_hole, &clip), subject);
  assert_eq!(xor(&expected_subject_with_hole, &clip), subject);

  assert_eq!(union(&expected_subject_with_hole, &subject), subject);
  assert_eq!(xor(&expected_subject_with_hole, &subject), clip);
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

  assert_eq!(intersection(&subject, &clip), subdivided_clip);
  assert_eq!(intersection(&clip, &subject), subdivided_clip);
  assert_eq!(union(&subject, &clip), subdivided_subject);
  assert_eq!(union(&clip, &subject), subdivided_subject);
  assert_eq!(
    difference(&subject, &clip),
    Polygon {
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
    }
  );
  assert_eq!(difference(&clip, &subject), Polygon { contours: vec![] });
}
