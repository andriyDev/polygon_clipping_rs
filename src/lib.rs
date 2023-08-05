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

#[cfg(test)]
mod tests;
