# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
"""
Regression tests for GUI bug fix #39: host requirements pre-fill from job template.

When a job template's steps declare hostRequirements, the host-requirements
tab should pre-select the custom radio and show the populated values.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from helpers import SAMPLE_TEMPLATE, SubmitterDialog


# A template with hostRequirements on the step
_TEMPLATE_WITH_HOST_REQS = {
    **SAMPLE_TEMPLATE,
    "steps": [
        {
            **SAMPLE_TEMPLATE["steps"][0],
            "hostRequirements": {
                "amounts": [
                    {"name": "amount.worker.vcpu", "min": 8, "max": 64},
                    {"name": "amount.worker.memory", "min": 16384},
                ],
                "attributes": [
                    {"name": "attr.worker.os.family", "anyOf": ["linux"]},
                ],
            },
        }
    ],
}


@pytest.fixture
def bundle_with_host_reqs(tmp_path) -> str:
    """Create a job bundle with hostRequirements in the template."""
    d = tmp_path / "bundle_host_reqs"
    d.mkdir()
    (d / "template.json").write_text(json.dumps(_TEMPLATE_WITH_HOST_REQS))
    return str(d)


class TestHostRequirementsPreFill:
    """#39: Host requirements should be pre-filled from the job template."""

    def test_custom_radio_selected_when_template_has_host_reqs(
        self, bundle_with_host_reqs, submitter_env
    ):
        """When the template declares hostRequirements, the 'custom' radio should be checked."""
        with SubmitterDialog.open(bundle_with_host_reqs, env=submitter_env) as app:
            app.wait_farm_resolved()
            app.activate_tab("Host requirements")
            custom_radio = app.locator(
                'radio_button[name="Run on worker hosts that meet the following requirements"]'
            )
            assert custom_radio.exists(), "Custom host requirements radio not found"
            elt = custom_radio.element()
            assert elt.checked, (
                "Custom radio should be checked when template has hostRequirements"
            )

    def test_default_radio_checked_when_no_host_reqs(
        self, bundle_dir, submitter_env
    ):
        """When the template has no hostRequirements, the 'all hosts' radio should be checked."""
        with SubmitterDialog.open(bundle_dir, env=submitter_env) as app:
            app.wait_farm_resolved()
            app.activate_tab("Host requirements")
            default_radio = app.locator(
                'radio_button[name="Run on all available worker hosts"]'
            )
            assert default_radio.exists()
            elt = default_radio.element()
            assert elt.checked, (
                "Default radio should be checked when template has no hostRequirements"
            )
