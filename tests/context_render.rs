use ezcmd::content::{all_recipes, all_tools};
use ezcmd::context::Context;
use ezcmd::os::Os;
use ezcmd::recommend;
use ezcmd::render::{self, RenderCtx};
use ezcmd::shell::Shell;

#[test]
fn compose_recipes_use_the_detected_package_manager_mapping() {
    let context = Context {
        git_repo: false,
        has_remote_origin: false,
        package_manager: "pnpm",
        ecosystems: vec![],
    };
    let frame = recommend::parse("create nextjs project");
    let recipe = all_recipes()
        .iter()
        .find(|recipe| recipe.id == "next.create")
        .expect("Next.js recipe");
    let mut output = Vec::new();
    render::render_recipe(
        &mut output,
        recipe,
        &frame,
        &context,
        Os::Linux,
        &RenderCtx {
            color: false,
            plain: true,
            width: 100,
            shell: Shell::Cmd,
        },
        false,
    )
    .expect("render");
    let output = String::from_utf8(output).expect("UTF-8 output");
    assert!(output.contains("pnpm dlx create-next-app@latest"));
    assert!(output.contains("pnpm dev"));
}

#[test]
fn workflows_fill_validated_repository_urls() {
    let context = Context {
        git_repo: false,
        has_remote_origin: false,
        package_manager: "npm",
        ecosystems: vec![],
    };
    let frame = recommend::parse("git init git@github.com:acme/demo.git");
    let tool = all_tools()
        .find(|tool| tool.slug == "git")
        .expect("Git tool");
    let section = tool
        .sections()
        .find(|section| section.title == "Initializing a Repo")
        .expect("init section");
    let mut output = Vec::new();
    render::render_workflow(
        &mut output,
        tool,
        section,
        &frame,
        &context,
        Os::Linux,
        &RenderCtx {
            color: false,
            plain: true,
            width: 100,
            shell: Shell::Cmd,
        },
    )
    .expect("render");
    let output = String::from_utf8(output).expect("UTF-8 output");
    assert!(output.contains("git remote add origin git@github.com:acme/demo.git"));
}
