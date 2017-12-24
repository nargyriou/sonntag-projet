io.stdout:setvbuf("no")
love.filesystem.setRequirePath("src/?.lua;src/?/init.lua")

local parser    = require "parser"
local Point     = require "point"
local Line      = require "line"
local Rectangle = require "rectangle"
local Polygon   = require "polygon"
local Sorter    = require "sorter"
local Movable   = require "movable"
local white = {255,255,255,255}
local black = {0,0,0,255}
local red   = {255,0,0,255}

local rawtype = type
function type(var)
	if rawtype(var) == "table" then
		return var.__type or "table"
	else
		return rawtype(var)
	end
end

local function generateRectangles(n)
	local maxx, maxy = love.graphics.getDimensions()
	local sorter = Sorter()
	for i=1, n do
		local width  = math.random(20, maxx*0.10)
		local height = math.random(0.2, 1)*width
		if math.random(1,10) > 5 then
			width, height = height, width
		end
		local x = math.random(0, maxx-width-10)
		local y = math.random(0, maxy-height-10)
		local r,g,b = math.random(0,255),math.random(0,255),math.random(0,255)
		if (r+g+b) > 612 then
			r,g,b = 0.8*r, 0.8*g, 0.8*b
		end
		local color = {r,g,b}
		local shape = Movable(Rectangle(x, y, width, height))
		shape.color = color
		sorter:addShape(shape)
	end

	return sorter
end

local function fetchSVG(file)
	local sorter = Sorter()
	local file = assert(io.open(file, "r"))
	local svg = file:read("*all")
	
	local rects = parser:parse(svg)

	for _, rect in pairs(rects) do
		local shape = Movable(rect)
		shape.color = {255, 0, 0}
		sorter:addShape(shape)
	end

	return sorter
end

function love.load(args)
	math.randomseed(os.time())
	love.window.setMode(900, 900, {vsync=false, centered=true})
	love.graphics.setBackgroundColor(white)
	love.graphics.setLineWidth(0.5)

	args[2] = args[2] or "svg/dessin_tetris1.svg"
	if args[2] then
		sorter = fetchSVG(args[2])
	end
end

function love.update(dt)
	if sorter then
		for i,v in pairs(sorter.shapes) do
			v:updatePos(dt)
		end
	end
end

local bascule
function love.draw()
	local height = love.graphics.getHeight()
	-- love.graphics.translate(0, height)
	-- love.graphics.rotate(-math.pi/2)
	if sorter then
		for i,v in pairs(sorter.shapes) do
			love.graphics.setColor(v.color)
			v:draw()
		end
	end
end

function love.keypressed(key, scancode, isrepeat)
	local dessins = {
		"svg/dessin_complex.svg", "svg/dessin_complex2.svg", "svg/dessin_grp.svg", 
		"svg/dessin_simple.svg", "svg/dessin_tetris1.svg", "svg/dessin_tetris2.svg"
	}

	if key == "return" then
		dessin = dessin or -1

		if dessin%2 == 1 then
			sorter:compact()
		else
			local tmp = (math.floor(dessin/2) % #dessins) + 1
			local file = dessins[tmp]
			print(tmp, file)
			sorter = fetchSVG(file)
		end

		dessin = dessin + 1
	elseif key == "space" then
		cycle = cycle or 1

		if cycle%3 == 1 then
			sorter = generateRectangles(100)
		elseif cycle%3 == 2 then
			sorter:sort()
		else
			sorter:compact()
		end

		cycle = cycle + 1
	end
end
