use std::path::PathBuf;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::clsi::{CompilePdfRequest, CompilePdfResponse};

#[derive(Clone)]
pub struct CompileHandler {
    work_dir: PathBuf,
}

impl CompileHandler {
    pub fn new() -> Self {
        let work_dir = std::env::var("CLSI_WORK_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".clsi")
            });
        Self { work_dir }
    }
}

impl CompileHandler {
    pub async fn compile_pdf(
        &self,
        request: Request<CompilePdfRequest>,
    ) -> Result<Response<CompilePdfResponse>, Status> {
        let req = request.into_inner();
        let _yjs_content = req.yjs_content;

        // TODO: Convert yjs_content to latex.
        // For now, use a sample exam paper template.
        let latex_content = r#"\documentclass[a3paper,12pt,twocolumn,landscape]{article}
\usepackage[margin=2cm,columnsep=1.5cm]{geometry}
\usepackage[fontset=none]{ctex}
\setCJKmainfont{Noto Sans CJK SC}
\setCJKsansfont{Noto Sans CJK SC}
\setCJKmonofont{Noto Sans Mono CJK SC}
\usepackage{unicode-math}
\setmainfont{Noto Serif}
\setsansfont{Noto Sans}
\setmonofont{Noto Sans Mono}
\setmathfont{Noto Sans Math}
\usepackage[shortlabels]{enumitem}
\usepackage{titlesec}

\titleformat{\section}{\large\bfseries}{第\chinese{section}题}{1em}{}
\titlespacing*{\section}{0pt}{1.5em}{0.8em}

\begin{document}

\begin{center}
  {\LARGE\bfseries 2026年高考数学模拟试卷}\\[0.5em]
  {\large 数学（理科）}\\[0.3em]
  \rule{\textwidth}{0.4pt}\\[0.3em]
  {\small 考试时间：120分钟 \quad 满分：150分}
\end{center}

\vspace{1em}

\noindent\textbf{一、选择题}（本大题共8小题，每小题5分，共40分。在每小题给出的四个选项中，只有一项是符合题目要求的。）

\vspace{0.5em}

\section{}
已知集合 $A = \{x \mid -1 < x < 2\}$，$B = \{x \mid x^2 - x - 6 < 0\}$，则 $A \cap B = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
若复数 $z$ 满足 $z(1+i) = 2i$，则 $|z| = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
已知向量 $\vec{a} = (1, 2)$，$\vec{b} = (x, -1)$，若 $\vec{a} \perp \vec{b}$，则 $x = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
函数 $f(x) = \ln x - \dfrac{1}{x}$ 的零点所在区间为 \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
已知等差数列 $\{a_n\}$ 中，$a_3 + a_7 = 10$，则 $S_9 = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
若 $\sin\alpha = \dfrac{3}{5}$，$\alpha \in \left(\dfrac{\pi}{2}, \pi\right)$，则 $\cos 2\alpha = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
已知双曲线 $\dfrac{x^2}{a^2} - \dfrac{y^2}{3} = 1\ (a > 0)$ 的离心率为 $2$，则 $a = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\section{}
函数 $f(x) = \dfrac{x^2 - 4}{|x - 2|}$ 的图象大致为 \rule{2cm}{0.4pt}

\vspace{1.5em}

\noindent\textbf{二、填空题}（本大题共6小题，每小题5分，共30分。）

\vspace{0.5em}

\noindent 9. 已知 $f(x) = x^3 - 3x$，则 $f(x)$ 的单调递减区间为 \rule{2cm}{0.4pt}

\vspace{0.5em}

\noindent 10. 若 $(1 + x)^{10}$ 的展开式中 $x^5$ 的系数为 \rule{2cm}{0.4pt}

\vspace{0.5em}

\noindent 11. 已知 $\triangle ABC$ 中，$a = 2$，$b = \sqrt{3}$，$B = 30°$，则 $A = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\noindent 12. 曲线 $y = xe^x$ 在点 $(1, e)$ 处的切线方程为 \rule{2cm}{0.4pt}

\vspace{0.5em}

\noindent 13. 若直线 $y = x + 1$ 与圆 $x^2 + y^2 = 4$ 相交于 $A$、$B$ 两点，则 $|AB| = $ \rule{2cm}{0.4pt}

\vspace{0.5em}

\noindent 14. 已知函数 $f(x) = \begin{cases} x^2 + 1, & x \geq 0 \\ -x + 1, & x < 0 \end{cases}$，若 $f(f(a)) = 5$，则 $a = $ \rule{2cm}{0.4pt}

\vspace{1.5em}

\noindent\textbf{三、解答题}（本大题共6小题，共80分。解答应写出文字说明、证明过程或演算步骤。）

\vspace{0.5em}

\noindent 15.（本小题满分12分）

已知数列 $\{a_n\}$ 满足 $a_1 = 1$，$a_{n+1} = 2a_n + 1$。

\begin{enumerate}[(1)]
  \item 证明：$\{a_n + 1\}$ 是等比数列；
  \item 求数列 $\{a_n\}$ 的前 $n$ 项和 $S_n$。
\end{enumerate}

\vspace{4em}

\noindent 16.（本小题满分12分）

在 $\triangle ABC$ 中，内角 $A$、$B$、$C$ 的对边分别为 $a$、$b$、$c$，已知 $2\sin A \cos B = \sin C$。

\begin{enumerate}[(1)]
  \item 求 $B$；
  \item 若 $a = 2$，$c = 3$，求 $\triangle ABC$ 的面积。
\end{enumerate}

\vspace{4em}

\noindent 17.（本小题满分14分）

如图，四棱锥 $P\text{-}ABCD$ 中，底面 $ABCD$ 为矩形，$PA \perp$ 底面 $ABCD$，$PA = AB = 1$，$AD = 2$。

\begin{enumerate}[(1)]
  \item 证明：$BC \perp PB$；
  \item 求直线 $PC$ 与平面 $PAB$ 所成角的正弦值。
\end{enumerate}

\vspace{4em}

\noindent 18.（本小题满分14分）

已知椭圆 $C: \dfrac{x^2}{4} + y^2 = 1$，点 $P(1, \frac{1}{2})$ 在椭圆上。

\begin{enumerate}[(1)]
  \item 求椭圆 $C$ 在点 $P$ 处的切线方程；
  \item 过点 $P$ 作直线 $l$ 与椭圆 $C$ 交于另一点 $Q$，若 $\triangle OPQ$ 的面积为 $\dfrac{\sqrt{3}}{4}$，求直线 $l$ 的方程。
\end{enumerate}

\vspace{4em}

\noindent 19.（本小题满分14分）

已知函数 $f(x) = e^x - ax\ (a \in \mathbb{R})$。

\begin{enumerate}[(1)]
  \item 讨论 $f(x)$ 的单调性；
  \item 若 $f(x) \geq 0$ 对所有 $x \in \mathbb{R}$ 成立，求 $a$ 的取值范围。
\end{enumerate}

\vspace{4em}

\noindent 20.（本小题满分14分）

设随机变量 $X \sim N(\mu, \sigma^2)$，已知 $P(X \leq \mu - \sigma) = 0.16$，$P(X \leq \mu + \sigma) = 0.84$。

\begin{enumerate}[(1)]
  \item 求 $P(|X - \mu| \leq 2\sigma)$；
  \item 若从该总体中随机抽取3个样本，求至少有2个样本落在 $(\mu - \sigma, \mu + \sigma)$ 内的概率。
\end{enumerate}

\end{document}
"#;

        // Create a unique working directory for this compilation.
        let job_id = uuid::Uuid::new_v4();
        let job_dir = self.work_dir.join(job_id.to_string());
        std::fs::create_dir_all(&job_dir).map_err(|e| {
            error!("Failed to create job dir {:?}: {}", job_dir, e);
            Status::internal(format!("Failed to create job dir: {}", e))
        })?;

        // Resolve symlinks (macOS /tmp → /private/tmp) so podman can mount.
        let job_dir_real = job_dir.canonicalize().map_err(|e| {
            error!("Failed to canonicalize {:?}: {}", job_dir, e);
            Status::internal(format!("Failed to resolve path: {}", e))
        })?;

        // Write latex source file.
        let tex_path = job_dir_real.join("input.tex");
        tokio::fs::write(&tex_path, latex_content)
            .await
            .map_err(|e| {
                error!("Failed to write tex file: {}", e);
                Status::internal(format!("Failed to write tex file: {}", e))
            })?;

        info!("Wrote tex file to {:?}", tex_path);

        // Run podman to compile latex → pdf.
        let job_dir_str = job_dir_real.to_string_lossy().to_string();
        let cmd = format!(
            "podman run --rm --entrypoint sh -v {}:/workspace pexm/latex -c 'cd /workspace && xelatex -interaction=nonstopmode input.tex'",
            job_dir_str,
        );

        info!("Running: {}", cmd);

        let output = tokio::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await
            .map_err(|e| {
                error!("Failed to execute podman: {}", e);
                Status::internal(format!("Failed to execute podman: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("xelatex failed:\nstdout: {}\nstderr: {}", stdout, stderr);

            // Clean up job dir.
            if let Err(e) = tokio::fs::remove_dir_all(&job_dir_real).await {
                error!("Failed to clean up job dir {:?}: {}", job_dir_real, e);
            }

            return Ok(Response::new(CompilePdfResponse {
                pdf_data: vec![],
                success: false,
                error: format!("xelatex compilation failed:\nstdout: {}\nstderr: {}", stdout, stderr),
            }));
        }

        // Read the generated PDF.
        let pdf_path = job_dir_real.join("input.pdf");
        let pdf_data = tokio::fs::read(&pdf_path)
            .await
            .map_err(|e| {
                error!("Failed to read pdf file {:?}: {}", pdf_path, e);
                Status::internal(format!("Failed to read pdf file: {}", e))
            })?;

        info!("Generated PDF, {} bytes", pdf_data.len());

        // Clean up job dir.
        if let Err(e) = tokio::fs::remove_dir_all(&job_dir_real).await {
            error!("Failed to clean up job dir {:?}: {}", job_dir_real, e);
        }

        Ok(Response::new(CompilePdfResponse {
            pdf_data,
            success: true,
            error: String::new(),
        }))
    }
}
