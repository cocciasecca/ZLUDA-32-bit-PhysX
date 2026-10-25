"""Build-state validation must fail before any device access."""

import unittest
import install_fixtures as fixtures

deploy = fixtures.module(
    "deploy_validation_test", fixtures.HERE.parent / "deploy_legion.py"
)


class BuildStateTests(unittest.TestCase):
    def info(self, **changes):
        return dict(headBranch=deploy.BRANCH, workflowName="ZLUDA", **changes)

    def test_active_runs_explain_wait_and_link(self):
        for status in ("queued", "in_progress", "waiting"):
            with self.assertRaisesRegex(
                RuntimeError, "non ancora terminata.*actions/runs/123"
            ):
                deploy.validate_run(self.info(status=status, conclusion=""), "123")

    def test_failed_and_wrong_branch_runs_are_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "esito failure"):
            deploy.validate_run(
                self.info(status="completed", conclusion="failure"), "123"
            )
        info = self.info(status="completed", conclusion="success")
        info["headBranch"] = "master"
        with self.assertRaisesRegex(RuntimeError, "non compatibile"):
            deploy.validate_run(info, "123")

    def test_successful_matching_run_is_accepted(self):
        deploy.validate_run(self.info(status="completed", conclusion="success"), "123")
