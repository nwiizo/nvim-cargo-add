local M = {}

local cargo_add = require("nvim_cargo_add")

function M.setup(opts)
	opts = opts or {}

	vim.api.nvim_create_user_command("CargoAdd", function(args)
		local parts = vim.split(args.args, " ")
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
		complete = function(ArgLead, CmdLine, CursorPos)
			local ok, results = pcall(cargo_add.search, ArgLead)
			if ok then
				return results
			end
			return {}
		end,
	})
end

return M
