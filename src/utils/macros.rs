// check if novalue and return something useful
//
// usage:
// 
// // instead of this
// let maybe = get_option();
// let maybe = match maybe {
//  Some(v) => v,
//  None    => return Err("oh no!"),
// };
//
// // do this(with Option)
// let maybe = null!(opt get_option(), Err("oh no!"));
// // and the same with Result
// let maybe = null!(res get_result(), Err("oh no!"));
#[macro_export]
macro_rules! null {
  // return result in Option
  (opt $e:expr, $r:expr) => {
    match $e {
      Some(v) => v,
      None    => return $r,
    }
  };

  // return result in Result, preprocess error
  (res $e:expr, $r:expr, $f:ident) => {
    match $e {
      Ok(v)  => v,
      Err(e) => {
        $f(e); 
        return $r;
      },
    }
  };
  (res $e:expr, $r:expr, $closure:tt) => {{
    let f = $closure;
    null!(res $e, $r, f)
  }};
  (res $e:expr, $r:expr) => { null!(res $e, $r, (|e| eprintln!("{:?}", e))) };
}


// http code on error
#[macro_export]
macro_rules! http_code {
  ($gen:ident opt $e:expr => $f:ident $code:expr) => { null!(opt $e, $f($gen($code))) };
  ($gen:ident opt $e:expr => $f:ident) => { http_code!($gen opt $e => $f 400) };
  ($gen:ident opt $e:expr => $code:expr) => { null!(opt $e, $gen($code)) };
  ($gen:ident opt $e:expr) => { http_code!($gen opt $e => 400) };

  ($gen:ident res $e:expr => $f:ident $code:expr) => { null!(res $e, $f($gen($code))) };
  ($gen:ident res $e:expr => $f:ident) => { http_code!($gen res $e => $f 400) };
  ($gen:ident res $e:expr => $code:expr) => { null!(res $e, $gen($code)) };
  ($gen:ident res $e:expr) => { http_code!($gen res $e => 400) };
}

