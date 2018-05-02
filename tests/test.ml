external rust_main: unit -> int option option option = "foo"
external map: 'a list -> 'a list = "caml_map"

open Format

let rec pp_list fmt sep ff = function
    | [] -> ()
    | [x] -> Format.fprintf ff "%a" fmt x
    | (hd :: tl) -> Format.fprintf ff "%a%s%a" fmt hd sep (pp_list fmt sep) tl


let () =
    match rust_main () with
      | None -> printf "None\n"
      | Some(None) -> printf "Some(None)\n"
      | Some(Some(None)) -> printf "Some(Some(None))\n"
      | Some(Some(Some(x))) -> printf "Some(Some(Some(%d)))\n" x ;

    let lst = [1 ; 2 ; 3] in

    Format.printf "%a\n"
        (pp_list (fun ff x -> Format.fprintf ff "%d" x) ", ") lst ;
    Format.printf "%a\n"
        (pp_list (fun ff x -> Format.fprintf ff "%d" x) ", ") (map lst) ;

    Format.printf "%a\n"
        (pp_list (fun ff x -> Format.fprintf ff "%d" x) ", ") (map []) ;

    ()
