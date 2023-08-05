use glam::Vec2;

use crate::{difference, intersection, union, xor, Polygon};

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
