local _src = debug.getinfo(1, "S").source
if _src:sub(1, 1) == "@" then
	_src = _src:sub(2)
end
local _dir = _src:match("(.*/)") or "./"
local data_raw = dofile(_dir .. "data_raw.lua")

local categories = {
	["item"] = "Item",
	["fluid"] = "Fluid",
	["virtual-signal"] = "Virtual",
}

local signals_by_category = {}
for cat, _ in pairs(categories) do
	signals_by_category[cat] = {}
end

for cat, _ in pairs(categories) do
	for name, proto in pairs(data_raw[cat] or {}) do
		if not (proto.hidden or (proto.hide_from_signal_gui == true)) then
			if not name:find("parameter") then
				table.insert(signals_by_category[cat], name)
			end
		end
	end
	table.sort(signals_by_category[cat])
end

local function to_rust_variant(name)
	name = name:gsub("-", "_")
	local parts = {}
	for part in name:gmatch("[^_]+") do
		if #part > 0 then
			table.insert(parts, part:sub(1, 1):upper() .. part:sub(2))
		end
	end
	local variant = table.concat(parts)
	if variant:match("^%d") then
		variant = "_" .. variant
	end
	if variant == "Self" or variant == "self" then
		variant = "SignalSelf"
	end
	return variant
end

local lines = {}

table.insert(lines, "use strum_macros::{EnumString, Display};")
table.insert(lines, "")

for raw_cat, enum_name in pairs(categories) do
	local names = signals_by_category[raw_cat]
	table.insert(lines, string.format("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]"))
	table.insert(lines, string.format("pub enum %s {", enum_name))

	for _, name in ipairs(names) do
		local variant = to_rust_variant(name)
		table.insert(lines, string.format("    %s,", variant))
	end

	table.insert(lines, "}")
	table.insert(lines, "")

	table.insert(lines, string.format("impl %s {", enum_name))
	table.insert(lines, string.format("    pub fn category(&self) -> String {"))
	table.insert(lines, string.format('        String::from("%s")', raw_cat))
	table.insert(lines, "    }")
	table.insert(lines, "}")
	table.insert(lines, "")
end

local file_content = table.concat(lines, "\n")
local output_path = _dir .. "../../src/lib/game/signals.rs"
local file = io.open(output_path, "w")

if not file then
	error("Could not open file for writing: " .. output_path)
end

file:write(file_content)
file:close()

print("Successfully wrote signals enums to " .. output_path)
