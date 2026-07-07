local function escape_latex(s)
	s = string.gsub(s, "\\", "\\textbackslash{}")
	s = string.gsub(s, "%%", "\\%%")
	s = string.gsub(s, "%$", "\\$")
	s = string.gsub(s, "&", "\\&")
	s = string.gsub(s, "#", "\\#")
	s = string.gsub(s, "_", "\\_")
	s = string.gsub(s, "%{", "\\{")
	s = string.gsub(s, "%}", "\\}")
	s = string.gsub(s, "%~", "\\textasciitilde{}")
	s = string.gsub(s, "%^", "\\textasciicircum{}")
	s = string.gsub(s, "%(%)", "(\\hspace{2em})")
	s = string.gsub(s, "（）", "(\\hspace{2em})")
	return s
end

local function render_paragraph(block)
	local parts = {}
	for _, child in ipairs(block.children or {}) do
		if child.type == "text" then
			table.insert(parts, escape_latex(child.text))
		elseif child.type == "mathInline" then
			table.insert(parts, "$" .. child.attrs["latex"] .. "$")
		end
	end

	return table.concat(parts)
end

local function render_list_item(block)
	local parts = {}

	for _, child in ipairs(block.children or {}) do
		if child.type == "paragraph" then
			local text = render_paragraph(child)
			if text ~= "" then
				table.insert(parts, text .. "\n")
			end
		end
	end

	return table.concat(parts)
end

local function render_uppercase_litter_list(block)
	local parts = {}
	local nums = 0
	for _, child in ipairs(block.children or {}) do
		if child.type ~= "listItem" then
			goto continue
		end

		local text = render_list_item(child)
		if text ~= "" then
			table.insert(parts, "\\task " .. text .. "\n")
			nums = nums + 1
		end

		::continue::
	end

	return "\\begin{tasks}(" .. nums .. ")\n" .. table.concat(parts) .. "\\end{tasks}\n"
end

local function render_ordered_list(block)
	local parts = {}
	for _, child in ipairs(block.children or {}) do
		if child.type ~= "listItem" then
			goto continue
		end

		local text = render_list_item(child)
		if text ~= "" then
			table.insert(parts, "\\item " .. text .. "\n")
		end

		::continue::
	end

	return "\\begin{enumerate}[itemsep=-0.3em,topsep=0pt,label=\\arabic*)]\n"
		.. table.concat(parts)
		.. "\\end{enumerate}\n"
end

local function render_bullet_list(block)
	local parts = {}
	for _, child in ipairs(block.children or {}) do
		if child.type ~= "listItem" then
			goto continue
		end

		local text = render_list_item(child)
		if text ~= "" then
			table.insert(parts, "\\item " .. text .. "\n")
		end

		::continue::
	end

	return "\\begin{itemize}[itemsep=-0.3em,topsep=0pt]\n" .. table.concat(parts) .. "\\end{itemize}\n"
end

local function render_question(block)
	local parts = {}

	for _, child in ipairs(block.children or {}) do
		local text = ""
		if child.type == "paragraph" then
			text = render_paragraph(child)
		elseif child.type == "uppercaseLetterList" then
			text = render_uppercase_litter_list(child)
		elseif child.type == "orderedList" then
			text = render_ordered_list(child)
		elseif child.type == "bulletList" then
			text = render_bullet_list(child)
		end
		if text ~= "" then
			table.insert(parts, text .. "\n")
		end
	end

	return "\\item " .. table.concat(parts)
end

local function render_block(block)
	if block.type ~= "doc" then
		return ""
	end

	local parts = {}

	local first_question = true
	for _, child in ipairs(block.children or {}) do
		if child.type == "paragraph" then
			local text = render_paragraph(child)
			if text ~= "" then
				table.insert(parts, text .. "\n")
			end
		elseif child.type == "question" then
			local text = render_question(child)
			if text ~= "" then
				if first_question then
					table.insert(parts, "\\begin{enumerate}[itemsep=0.2em,topsep=0pt] ")
					first_question = false
				else
					table.insert(parts, "\\begin{enumerate}[itemsep=0.2em,topsep=0pt,resume] ")
				end
				table.insert(parts, text .. "\n")
				table.insert(parts, "\\end{enumerate}\n")
			end
		end
	end

	return table.concat(parts)
end

function render_latex(block, config)
	config = config or {}
	local title = config.title or "试卷"
	-- local subject = config.subject or ""

	local body = render_block(block)
	if body == "" then
		body = "\\textit{（暂无内容）}"
	end

	local tex = "\\documentclass[11pt,space]{ctexart}\n"
	tex = tex
		.. "\\usepackage[a3paper,twocolumn,landscape,hmargin={3.5cm,1.3cm},vmargin={1.5cm,1.5cm},footskip=0.75cm,headsep=0.25cm]{geometry}\n"
	tex = tex .. "\\setlength{\\columnsep}{1.8cm}\n"

	tex = tex .. "\\def\\ExamPrimaryTitle#1{\\begin{center}{\\zihao{3}#1}\\end{center}}\n"
	tex = tex .. "\\def\\ExamSecondaryTitle#1{\\begin{center}{\\zihao{2}\\heiti#1}\\end{center}}\n"
	tex = tex .. "\\RequirePackage{titlesec}\n"
	tex = tex .. "\\titlespacing*{\\section}{0pt}{9pt}{4pt}\n"

	tex = tex .. "\\zihao{5}\n"

	tex = tex
		.. "\\ctexset{section = {name = {,、\\hspace*{-5mm}},number = \\chinese{section},format = {\\zihao{5}\\heiti\\raggedright}}}\n"
	tex = tex
		.. "\\ctexset{subsection = {name = {\\hspace{5mm},\\hspace*{-1mm}},number = (\\chinese{subsection}),format = {\\zihao{5}\\heiti\\raggedright}}}\n"

	tex = tex .. "\\usepackage{tasks}\n"
	tex = tex .. "\\settasks{label={\\Alph*.},\n"
	tex = tex .. "label-offset={0.4em},\n"
	tex = tex .. "label-align=left,\n"
	tex = tex .. "column-sep={2pt},\n"
	tex = tex .. "item-indent={2em},\n"
	tex = tex .. "before-skip={-0.7em},\n"
	tex = tex .. "after-skip={-0.7em}}\n"

	tex = tex .. "\\newfontfamily\\TNRBI{Noto Sans Math}\n"
	tex = tex .. "\\usepackage{enumitem}\n"
	tex = tex .. "\\usepackage{tasks}\n"
	tex = tex .. "\\usepackage{graphicx}\n"
	tex = tex .. "\\usepackage{array}\n"
	tex = tex .. "\\usepackage{multirow}\n"
	tex = tex .. "\\usepackage{float}\n"
	tex = tex .. "\\usepackage{amsmath}\n"
	tex = tex .. "\\usepackage{adjustbox}\n"
	tex = tex .. "\\setmainfont{Noto Sans Mono CJK TC}\n"
	tex = tex .. "\\pagestyle{empty}\n"

	tex = tex .. "\\usepackage{fontspec}\n"
	tex = tex .. "\\usepackage{unicode-math}\n"
	tex = tex .. "\\setCJKmainfont{Noto Sans CJK SC}\n"
	tex = tex .. "\\setCJKsansfont{Noto Sans CJK SC}\n"
	tex = tex .. "\\setCJKmonofont{Noto Sans Mono CJK SC}\n"
	tex = tex .. "\\setmainfont{Noto Sans}\n"
	tex = tex .. "\\setsansfont{Noto Sans}\n"
	tex = tex .. "\\setmonofont{Noto Sans Mono CJK SC}\n"
	tex = tex .. "\\setmathfont{Noto Sans Math}\n"

	tex = tex .. "\\begin{document}\n\n"
	if title and title ~= "" then
		tex = tex .. "\\ExamPrimaryTitle{" .. escape_latex(title) .. "}\n\n"
	end
	tex = tex .. body .. "\n\n"
	tex = tex .. "\\end{document}\n"
	return tex
end
