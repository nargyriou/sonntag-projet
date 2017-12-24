local Shape = require "shape"
local Point  = require "point"
local Line   = require "line"

local Rectangle = Shape:extend()

function Rectangle:new(x, y, width, height)
	self.attached = {}
	self.shape = "Rectangle"
	self.x = x
	self.y = y
	self.width  = width
	self.height = height

	return self
end

function Rectangle:getVertices()
	self.vertices = {}
	self.vertices[1] = Point(self.x, self.y)
	self.vertices[2] = Point(self.x + self.width, self.y)
	self.vertices[3] = Point(self.x + self.width, self.y+self.height)
	self.vertices[4] = Point(self.x, self.y+self.height)
	return self.vertices
end

function Rectangle:getEdges()
	local vertice = self:getVertices()
	self.edges = {}
	self.edges[1] = Line(vertice[1], vertice[2])
	self.edges[2] = Line(vertice[2], vertice[3])
	self.edges[3] = Line(vertice[3], vertice[4])
	self.edges[4] = Line(vertice[4], vertice[1])
	return self.edges
end

function Rectangle:move(x, y)
	local offx = x - self.x
	local offy = y - self.y
	for i,v in pairs(self.attached) do
		v:add(offx, offy)
	end
	self.x = x
	self.y = y
end

function Rectangle:left(val)
	self.x = self.x - val
end
function Rectangle:right(val)
	self.x = self.x + val
end
function Rectangle:up(val)
	self.y = self.y - val
end
function Rectangle:down(val)
	self.y = self.y + val
end

function Rectangle:draw()
	local edges = self:getEdges()
	for i,v in pairs(edges) do
		v:draw()
	end
	for i,v in pairs(self.attached) do
		v:draw()
	end
end

function Rectangle:getArea()
	return self.width*self.height
end

-- Prend une ou plusieurs shapes avec lesquelles on vérifie les collisions
-- Et (optionnel) un couple x,y pour vérifier une collision future si l'objet
-- se déplace sur ces coordonnées
function Rectangle:collide(shapes, x, y)
	-- Collision avec rien toujours fausse
	if not shapes then
		return false
	end

	-- Si on nous donne une shape unique on la met dans un array vide
	if type(shapes) == "object" then
		shapes = {shapes}
	end

	-- Si pas précisés on utilise les x,y de l'objet
	if not x or not y then
		x = self.x
		y = self.y
	end

	-- On traite les shapes maintenant
	for i,rect2 in pairs(shapes) do
		if (x <= rect2.x + rect2.width and
			x + self.width >= rect2.x and
			y <= rect2.y + rect2.height and
			self.height + y >= rect2.y) then
			return i
		end
	end

	return false
end

return Rectangle