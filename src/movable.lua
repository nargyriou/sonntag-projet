local Object = require "classic"
local Movable = Object:extend()

function Movable:new(shape)
	self.shape = shape
	for i,v in pairs(shape) do
		if self[i] == nil then
			self[i] = self.shape[i]
		end
	end

	self.timeElapsed = 0
	self.newx  = self.x
	self.oldx  = self.x
	self.newy  = self.y
	self.oldy  = self.y
	self.duration = 2

	self.tween = function(t)
		return math.sin( (t/2)*math.pi )
	end


	return self
end

function Movable:getArea()
	return self.shape:getArea()
end

function Movable:draw()
	return self.shape:draw()
end

function Movable:move(x, y)
	self.x = x
	self.y = y
	self.shape:move(x, y)
	self.travelling = false
end

function Movable:moveTo(x, y, duration)
	self.oldx = self.x
	self.oldy = self.y
	self.newx = x
	self.newy = y
	self.timeElapsed = 0
	self.duration = duration or 2
	self.travelling = true
end

function Movable:collide(...)
	return self.shape:collide(...)
end

function Movable:updatePos(dt)
	if not self.travelling then
		return
	end
	if self.timeElapsed >= self.duration then
		self.travelling = false
		return
	end
	self.timeElapsed = math.min(self.timeElapsed+dt, self.duration)
	self.completion  = self.timeElapsed / self.duration

	local movx = (self.newx - self.oldx) * self.tween(self.completion)
	local movy = (self.newy - self.oldy) * self.tween(self.completion)

	self:move(self.oldx + movx, self.oldy + movy)
	self.travelling = true
end

return Movable