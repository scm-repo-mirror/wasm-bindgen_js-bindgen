(module
	(import "js_sys" "const" (table $js_sys.externref.const 1 1 externref))
	(table $js_sys.externref.table 0 externref)
	(func $js_sys.externref.grow (param $size i32) (result i32)
		ref.null extern
		local.get $size
		table.grow $js_sys.externref.table
	)
	(func $js_sys.externref.get (param $index i32) (result externref)
		local.get $index
		i32.const 0
		i32.ge_s
		if (result externref)
			local.get $index
			table.get $js_sys.externref.table
		else
			local.get $index
			i32.const -1
			i32.mul
			i32.const 1
			i32.sub
			table.get $js_sys.externref.const
		end
	)
	(func $js_sys.externref.remove (param $index i32)
		local.get $index
		ref.null extern
		table.set $js_sys.externref.table
	)
)
