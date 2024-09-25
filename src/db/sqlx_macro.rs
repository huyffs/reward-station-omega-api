macro_rules! must_bind {
  ($sep:ident, $col:literal $op:tt $val:expr) => {
    $sep
      .push(format!(" {} {} ", $col, stringify!($op)))
      .push_bind_unseparated($val)
  };
}

macro_rules! maybe_bind {
  ($sep:ident, $col:literal $op:tt $maybe_val:expr) => {
    if let Some(val) = $maybe_val {
      $sep
        .push(format!(" {} {} ", $col, stringify!($op)))
        .push_bind_unseparated(val)
    } else {
      &$sep
    }
  };
  ($sep:ident, $col:literal $op:tt $maybe_val:expr, $mapper:expr) => {
    if let Some(val) = $maybe_val {
      let val = $mapper(val);
      $sep
        .push(format!(" {} {} ", $col, stringify!($op)))
        .push_bind_unseparated(val)
    } else {
      &$sep
    }
  };
}

macro_rules! offset_limit {
  ($query:ident, $offset:expr, $limit:expr) => {
    $query.push(" OFFSET ").push($offset);
    $query.push(" LIMIT ").push($limit);
  };
}

pub(crate) use maybe_bind;
pub(crate) use must_bind;
pub(crate) use offset_limit;
