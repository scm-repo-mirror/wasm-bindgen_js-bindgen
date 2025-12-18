(module
	(import "web_sys" "console.log"
		(func $web_sys.import.console.log (param externref)))
	(import "js_sys" "js_sys.externref.get"
		(func $js_sys.externref.get (param i64) (result externref)))
	(func $web_sys.console.log (param $index i64)
		local.get $index
		call $js_sys.externref.get
		call $web_sys.import.console.log
	)
)
