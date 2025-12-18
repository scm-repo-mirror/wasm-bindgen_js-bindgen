(module
	(import "js_sys" "const" (table $js_sys.externref.const i64 1 1 externref))
	(table $js_sys.externref.table i64 0 externref)
	(func $js_sys.externref.grow (param $size i64) (result i64)
		ref.null extern
		local.get $size
		table.grow $js_sys.externref.table
	)
	(func $js_sys.externref.get (param $index i64) (result externref)
		local.get $index
		i64.const 0
		i64.ge_s
		if (result externref)
			local.get $index
			table.get $js_sys.externref.table
		else
			local.get $index
			i64.const -1
			i64.mul
			i64.const 1
			i64.sub
			table.get $js_sys.externref.const
		end
	)
	(func $js_sys.externref.remove (param $index i64)
		local.get $index
		ref.null extern
		table.set $js_sys.externref.table
	)
)
