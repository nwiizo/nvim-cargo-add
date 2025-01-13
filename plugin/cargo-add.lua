if vim.fn.has("nvim-0.9") == 0 then
	vim.api.nvim_echo({
		{ "nvim-cargo-add requires at least nvim-0.9", "ErrorMsg" },
		{ "Please upgrade your neovim version", "WarningMsg" },
		{ "Press any key to exit", "ErrorMsg" },
	}, true, {})
	vim.fn.getchar()
	vim.cmd([[quit]])
end

if vim.g.loaded_cargo_add ~= nil then
	return
end

vim.g.loaded_cargo_add = 1

-- Initialize the plugin
local ok, cargo_add = pcall(require, "nvim_cargo_add")
if not ok then
	vim.api.nvim_err_writeln("Failed to load nvim-cargo-add: " .. cargo_add)
	return
end

-- Command definitions
local api = vim.api

api.nvim_create_user_command("CargoAdd", function(opts)
	local parts = vim.split(opts.args, " ")
	local crate_name = parts[1]
	local version = parts[2]

	if not crate_name then
		vim.notify("Usage: CargoAdd <crate_name> [version]", vim.log.levels.ERROR)
		return
	end

	local ok, err = pcall(cargo_add.add, crate_name, version)
	if not ok then
		vim.notify("Failed to add crate: " .. tostring(err), vim.log.levels.ERROR)
	else
		vim.notify("Successfully added " .. crate_name, vim.log.levels.INFO)
	end
end, {
	nargs = "*",
	complete = function(ArgLead)
		local ok, results = pcall(cargo_add.search, ArgLead)
		if ok then
			return results
		end
		return {}
	end,
	desc = "Add a crate to Cargo.toml",
})
