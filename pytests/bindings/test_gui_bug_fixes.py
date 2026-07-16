# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
"""
Regression tests for GUI bug fixes #39 and #56.

- HostRequirements.from_dict roundtrip (#39)
- _resolve_template_host_requirements logic (#39)
- _sync_config / _config_tracks_global (#56 — tested via xa11y, not here)
"""

import pytest

from deadline.client.ui.dataclasses import HostRequirements
from deadline.client.ui.job_bundle_submitter import _resolve_template_host_requirements


# --- #39: HostRequirements.from_dict ---


class TestHostRequirementsFromDict:
    """HostRequirements.from_dict roundtrip — proves the method works end-to-end."""

    def test_roundtrip_well_known_capabilities(self):
        """from_dict then serialize reproduces the input for well-known capabilities."""
        source = {
            "attributes": [
                {"name": "attr.worker.os.family", "anyOf": ["windows"]},
                {"name": "attr.worker.cpu.arch", "anyOf": ["x86_64"]},
            ],
            "amounts": [
                {"name": "amount.worker.vcpu", "min": 8, "max": 64},
                {"name": "amount.worker.memory", "min": 16384, "max": 131072},
            ],
        }
        result = HostRequirements.from_dict(source).serialize()
        # Compare amounts by name
        for amount in source["amounts"]:
            ref = next(a for a in result["amounts"] if a["name"] == amount["name"])
            assert ref.get("min") == amount.get("min")
            assert ref.get("max") == amount.get("max")
        # Compare attributes by name
        for attribute in source["attributes"]:
            ref = next(a for a in result["attributes"] if a["name"] == attribute["name"])
            assert set(ref.get("anyOf", ref.get("allOf", []))) == set(
                attribute.get("anyOf", attribute.get("allOf", []))
            )


# --- #39: _resolve_template_host_requirements ---


_HR_A = {
    "amounts": [{"name": "amount.worker.vcpu", "min": 8, "max": 64}],
    "attributes": [{"name": "attr.worker.os.family", "anyOf": ["linux"]}],
}
_HR_B = {
    "amounts": [{"name": "amount.worker.vcpu", "min": 1}],
}


class TestResolveTemplateHostRequirements:
    """Tests for _resolve_template_host_requirements prefill logic."""

    def test_no_steps_returns_none(self):
        assert _resolve_template_host_requirements({}) is None
        assert _resolve_template_host_requirements({"steps": []}) is None

    def test_single_step_with_requirements_prefills(self):
        template = {"steps": [{"name": "Step1", "hostRequirements": _HR_A}]}
        assert _resolve_template_host_requirements(template) == _HR_A

    def test_single_step_without_requirements_returns_none(self):
        template = {"steps": [{"name": "Step1"}]}
        assert _resolve_template_host_requirements(template) is None

    def test_multiple_steps_identical_requirements_prefills(self):
        template = {
            "steps": [
                {"name": "Step1", "hostRequirements": _HR_A},
                {"name": "Step2", "hostRequirements": dict(_HR_A)},
            ]
        }
        assert _resolve_template_host_requirements(template) == _HR_A

    def test_multiple_steps_different_requirements_returns_none(self):
        template = {
            "steps": [
                {"name": "Step1", "hostRequirements": _HR_A},
                {"name": "Step2", "hostRequirements": _HR_B},
            ]
        }
        assert _resolve_template_host_requirements(template) is None

    def test_some_steps_missing_requirements_returns_none(self):
        template = {
            "steps": [
                {"name": "Step1", "hostRequirements": _HR_A},
                {"name": "Step2"},
            ]
        }
        assert _resolve_template_host_requirements(template) is None
