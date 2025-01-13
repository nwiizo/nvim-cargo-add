local M = {}

local function load_cargo_add()
	local plugin_dir = vim.fn.fnamemodify(vim.fn.resolve(debug.getinfo(1, "S").source:sub(2)), ":h:h:h")
	local lib_path = plugin_dir .. "/target/release/libnvim_cargo_add.dylib"

	if vim.fn.filereadable(lib_path) == 0 then
		error("Cargo add library not found at: " .. lib_path)
	end

	local loaded = package.loadlib(lib_path, "luaopen_nvim_cargo_add")
	if loaded == nil then
		error("Failed to load library: " .. lib_path)
	end

	local cargo_add = loaded()
	if cargo_add == nil then
		error("Failed to initialize cargo_add module")
	end

	return cargo_add
end

local function create_float_window()
	local width = 100
	local height = 20
	local bufnr = vim.api.nvim_create_buf(false, true)
	local win_opts = {
		relative = "editor",
		width = width,
		height = height,
		col = (vim.o.columns - width) / 2,
		row = (vim.o.lines - height) / 2,
		style = "minimal",
		border = "rounded",
	}
	local winnr = vim.api.nvim_open_win(bufnr, true, win_opts)
	return bufnr, winnr
end

function M.setup(opts)
	opts = opts or {}
	opts.auto_format = opts.auto_format ~= false
	opts.float_preview = opts.float_preview ~= false

	local cargo_add
	local ok
	ok, cargo_add = pcall(load_cargo_add)
	if not ok then
		vim.notify("Failed to load cargo add library: " .. tostring(cargo_add), vim.log.levels.ERROR)
		return
	end

	if cargo_add == nil then
		vim.notify("Cargo add module is nil after loading", vim.log.levels.ERROR)
		return
	end

	-- CargoAdd command
	vim.api.nvim_create_user_command("CargoAdd", function(args)
		local parts = vim.split(args.args, " ")
		local crate_name = parts[1]
		local version = parts[2]

		if not crate_name then
			vim.notify("Usage: CargoAdd <crate_name> [version]", vim.log.levels.ERROR)
			return
		end

		local ok, err = pcall(cargo_add.add, crate_name, version, opts.auto_format)
		if not ok then
			vim.notify("Failed to add crate: " .. tostring(err), vim.log.levels.ERROR)
		else
			vim.notify("Successfully added " .. crate_name, vim.log.levels.INFO)
		end
	end, {
		nargs = "*",
		complete = function(ArgLead)
			if cargo_add and cargo_add.search then
				local ok, results = pcall(cargo_add.search, ArgLead)
				if ok then
					if opts.float_preview then
						local bufnr, winnr = create_float_window()
						vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, results)
						vim.api.nvim_create_autocmd("CmdlineLeave", {
							callback = function()
								if vim.api.nvim_win_is_valid(winnr) then
									vim.api.nvim_win_close(winnr, true)
								end
							end,
							once = true,
						})
					end
					return vim.tbl_map(function(result)
						return vim.split(result, " ")[1]
					end, results)
				end
			end
			return {}
		end,
	})

	-- CargoRemove command
	vim.api.nvim_create_user_command("CargoRemove", function(args)
		local crate_name = args.args

		if not crate_name or crate_name == "" then
			vim.notify("Usage: CargoRemove <crate_name>", vim.log.levels.ERROR)
			return
		end

		local ok, err = pcall(cargo_add.remove, crate_name, opts.auto_format)
		if not ok then
			vim.notify("Failed to remove crate: " .. tostring(err), vim.log.levels.ERROR)
		else
			vim.notify("Successfully removed " .. crate_name, vim.log.levels.INFO)
		end
	end, {
		nargs = 1,
		complete = function(ArgLead)
			local ok, deps = pcall(cargo_add.list)
			if ok then
				return vim.tbl_filter(function(dep)
					local name = vim.split(dep, " ")[1]
					return name:lower():find(ArgLead:lower(), 1, true)
				end, deps)
			end
			return {}
		end,
	})

	-- CargoAddDev command
	vim.api.nvim_create_user_command("CargoAddDev", function(args)
		local parts = vim.split(args.args, " ")
		local crate_name = parts[1]
		local version = parts[2]

		if not crate_name then
			vim.notify("Usage: CargoAddDev <crate_name> [version]", vim.log.levels.ERROR)
			return
		end

		local ok, err = pcall(cargo_add.add_dev, crate_name, version, opts.auto_format)
		if not ok then
			vim.notify("Failed to add dev dependency: " .. tostring(err), vim.log.levels.ERROR)
		else
			vim.notify("Successfully added " .. crate_name .. " as dev dependency", vim.log.levels.INFO)
		end
	end, {
		nargs = "*",
		complete = function(ArgLead)
			if cargo_add and cargo_add.search then
				local ok, results = pcall(cargo_add.search, ArgLead)
				if ok then
					if opts.float_preview then
						local bufnr, winnr = create_float_window()
						vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, results)
						vim.api.nvim_create_autocmd("CmdlineLeave", {
							callback = function()
								if vim.api.nvim_win_is_valid(winnr) then
									vim.api.nvim_win_close(winnr, true)
								end
							end,
							once = true,
						})
					end
					return vim.tbl_map(function(result)
						return vim.split(result, " ")[1]
					end, results)
				end
			end
			return {}
		end,
	})

	-- CargoDeps command
	vim.api.nvim_create_user_command("CargoDeps", function()
		local ok, deps = pcall(cargo_add.list)
		if not ok then
			vim.notify("Failed to list dependencies: " .. tostring(deps), vim.log.levels.ERROR)
			return
		end

		if #deps > 0 then
			if opts.float_preview then
				local bufnr, winnr = create_float_window()
				vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, deps)
				vim.api.nvim_buf_set_option(bufnr, "modifiable", false)
				vim.api.nvim_create_autocmd("BufLeave", {
					buffer = bufnr,
					callback = function()
						if vim.api.nvim_win_is_valid(winnr) then
							vim.api.nvim_win_close(winnr, true)
						end
					end,
					once = true,
				})
			else
				vim.notify("Current dependencies:", vim.log.levels.INFO)
				for _, dep in ipairs(deps) do
					vim.notify("  " .. dep, vim.log.levels.INFO)
				end
			end
		else
			vim.notify("No dependencies found", vim.log.levels.INFO)
		end
	end, {})

	-- CargoAddDebug command
	vim.api.nvim_create_user_command("CargoAddDebug", function()
		local plugin_dir = debug.getinfo(1, "S").source:sub(2)
		plugin_dir = vim.fn.fnamemodify(plugin_dir, ":h:h:h")
		local lib_path = plugin_dir .. "/target/release/libnvim_cargo_add.dylib"

		print("Current directory:", vim.fn.getcwd())
		print("Plugin directory:", plugin_dir)
		print("Library path:", lib_path)
		print("Library exists:", vim.fn.filereadable(lib_path))
		print("Lua package path:", package.path)
		print("Lua package cpath:", package.cpath)

		print("Cargo Add object:", vim.inspect(cargo_add))
		if cargo_add then
			print("Available functions:", vim.inspect(vim.tbl_keys(cargo_add)))
		end
	end, {})
end

return M
