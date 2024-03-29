// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Diora Team
/// @title Pallet Parachain Staking Interface
/// @dev The interface through which solidity contracts will interact with Parachain Staking
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000800
interface ParachainStaking {
    /// @dev Check whether the specified address is currently a staking delegator
    /// @custom:selector fd8ab482
    /// @param The delegator address in form of 20 or 32 hex bytes
    /// @return A boolean confirming whether the address is a delegator
    function isDelegator(bytes memory delegator) external view returns (bool);

    /// @dev Check whether the specified address is currently a collator candidate
    /// @custom:selector d51b9e93
    /// @param The candidate address in form of 32 hex bytes
    /// @return A boolean confirming whether the address is a collator candidate
    function isCandidate(bytes32 candidate) external view returns (bool);

    /// @dev Check whether the specifies address is currently a part of the active set
    /// @custom:selector 740d7d2a
    /// @param The candidate address in form of 32 hex bytes
    /// @return A boolean confirming whether the address is a part of the active set
    function isSelectedCandidate(bytes32 candidate)
    external
    view
    returns (bool);

    /// @dev Total points awarded to all collators in a particular round
    /// @custom:selector 9799b4e7
    /// @param round the round for which we are querying the points total
    /// @return The total points awarded to all collators in the round
    function points(uint32 round) external view returns (uint256);

    /// @dev Total points awarded to a specific collator in a particular round.
    /// A value of `0` may signify that no blocks were produced or that the storage for that round has been removed
    /// @custom:selector bfea66ac
    /// @param round the round for which we are querying the awarded points
    /// @param candidate The candidate to whom the points are awarded
    /// @return The total points awarded to the collator for the provided round
    function awardedPoints(uint32 round, bytes32 candidate)
    external
    view
    returns (uint32);

    /// @dev The amount delegated in support of the candidate by the delegator
    /// @custom:selector a73e51bc
    /// @param delegator Who made this delegation
    /// @param candidate The candidate for which the delegation is in support of
    /// @return The amount of the delegation in support of the candidate by the delegator
    function delegationAmount(bytes memory delegator, bytes32 candidate)
    external
    view
    returns (uint256);

    /// @dev Whether the delegation is in the top delegations
    /// @custom:selector 91cc8657
    /// @param delegator Who made this delegation
    /// @param candidate The candidate for which the delegation is in support of
    /// @return If delegation is in top delegations (is counted)
    function isInTopDelegations(bytes memory delegator, bytes32 candidate)
    external
    view
    returns (bool);

    /// @dev Get the minimum delegation amount
    /// @custom:selector 02985992
    /// @return The minimum delegation amount
    function minDelegation() external view returns (uint256);

    /// @dev Get the CandidateCount weight hint
    /// @custom:selector a9a981a3
    /// @return The CandidateCount weight hint
    function candidateCount() external view returns (uint256);

    /// @dev Get the current round number
    /// @custom:selector 146ca531
    /// @return The current round number
    function round() external view returns (uint256);

    /// @dev Get the CandidateDelegationCount weight hint
    /// @custom:selector 2ec087eb
    /// @param candidate The address for which we are querying the nomination count
    /// @return The number of nominations backing the collator
    function candidateDelegationCount(bytes32 candidate)
    external
    view
    returns (uint32);

    /// @dev Get the CandidateAutoCompoundingDelegationCount weight hint
    /// @custom:selector 905f0806
    /// @param candidate The address for which we are querying the auto compounding
    ///     delegation count
    /// @return The number of auto compounding delegations
    function candidateAutoCompoundingDelegationCount(bytes32 candidate)
    external
    view
    returns (uint32);

    /// @dev Get the DelegatorDelegationCount weight hint
    /// @custom:selector 067ec822
    /// @param delegator The address for which we are querying the delegation count
    /// @return The number of delegations made by the delegator
    function delegatorDelegationCount(bytes memory delegator)
    external
    view
    returns (uint256);

    /// @dev Get the selected candidates for the current round
    /// @custom:selector bcf868a6
    /// @return The selected candidate accounts
    function selectedCandidates() external view returns (bytes32[] memory);

    /// @dev Whether there exists a pending request for a delegation made by a delegator
    /// @custom:selector 3b16def8
    /// @param delegator the delegator that made the delegation
    /// @param candidate the candidate for which the delegation was made
    /// @return Whether a pending request exists for such delegation
    function delegationRequestIsPending(bytes memory delegator, bytes32 candidate)
    external
    view
    returns (bool);

    /// @dev Whether there exists a pending exit for candidate
    /// @custom:selector 43443682
    /// @param candidate the candidate for which the exit request was made
    /// @return Whether a pending request exists for such delegation
    function candidateExitIsPending(bytes32 candidate)
    external
    view
    returns (bool);

    /// @dev Whether there exists a pending bond less request made by a candidate
    /// @custom:selector d0deec11
    /// @param candidate the candidate which made the request
    /// @return Whether a pending bond less request was made by the candidate
    function candidateRequestIsPending(bytes32 candidate)
    external
    view
    returns (bool);

    /// @dev Returns the percent value of auto-compound set for a delegation
    /// @custom:selector b4d4c7fd
    /// @param delegator the delegator that made the delegation
    /// @param candidate the candidate for which the delegation was made
    /// @return Percent of rewarded amount that is auto-compounded on each payout
    function delegationAutoCompound(bytes memory delegator, bytes32 candidate)
    external
    view
    returns (uint8);

    /// @dev Make a delegation in support of a collator candidate
    /// @custom:selector 829f5ee3
    /// @param candidate The address of the supported collator candidate
    /// @param amount The amount bonded in support of the collator candidate
    /// @param candidateDelegationCount The number of delegations in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function delegate(
        bytes32 candidate,
        uint256 amount,
        uint32 candidateDelegationCount,
        uint32 delegatorDelegationCount
    ) external;

    /// @dev Make a delegation in support of a collator candidate
    /// @custom:selector 4b8bc9bf
    /// @param candidate The address of the supported collator candidate
    /// @param amount The amount bonded in support of the collator candidate
    /// @param autoCompound The percent of reward that should be auto-compounded
    /// @param candidateDelegationCount The number of delegations in support of the candidate
    /// @param candidateAutoCompoundingDelegationCount The number of auto-compounding delegations
    /// in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function delegateWithAutoCompound(
        bytes32 candidate,
        uint256 amount,
        uint8 autoCompound,
        uint32 candidateDelegationCount,
        uint32 candidateAutoCompoundingDelegationCount,
        uint32 delegatorDelegationCount
    ) external;

    /// @notice DEPRECATED use batch util with scheduleRevokeDelegation for all delegations
    /// @dev Request to leave the set of delegators
    /// @custom:selector f939dadb
    function scheduleLeaveDelegators() external;

    /// @notice DEPRECATED use batch util with executeDelegationRequest for all delegations
    /// @dev Execute request to leave the set of delegators and revoke all delegations
    /// @custom:selector fb1e2bf9
    /// @param delegator The leaving delegator
    /// @param delegatorDelegationCount The number of active delegations to be revoked by delegator
    function executeLeaveDelegators(
        bytes memory delegator,
        uint32 delegatorDelegationCount
    ) external;

    /// @notice DEPRECATED use batch util with cancelDelegationRequest for all delegations
    /// @dev Cancel request to leave the set of delegators
    /// @custom:selector f7421284
    function cancelLeaveDelegators() external;

    /// @dev Request to revoke an existing delegation
    /// @custom:selector 1a1c740c
    /// @param candidate The address of the collator candidate which will no longer be supported
    function scheduleRevokeDelegation(bytes32 candidate) external;

    /// @dev Bond more for delegators with respect to a specific collator candidate
    /// @custom:selector 0465135b
    /// @param candidate The address of the collator candidate for which delegation shall increase
    /// @param more The amount by which the delegation is increased
    function delegatorBondMore(bytes32 candidate, uint256 more) external;

    /// @dev Request to bond less for delegators with respect to a specific collator candidate
    /// @custom:selector c172fd2b
    /// @param candidate The address of the collator candidate for which delegation shall decrease
    /// @param less The amount by which the delegation is decreased (upon execution)
    function scheduleDelegatorBondLess(bytes32 candidate, uint256 less)
    external;

    /// @dev Execute pending delegation request (if exists && is due)
    /// @custom:selector e98c8abe
    /// @param delegator The address of the delegator
    /// @param candidate The address of the candidate
    function executeDelegationRequest(bytes memory delegator, bytes32 candidate)
    external;

    /// @dev Cancel pending delegation request (already made in support of input by caller)
    /// @custom:selector c90eee83
    /// @param candidate The address of the candidate
    function cancelDelegationRequest(bytes32 candidate) external;

    /// @dev Sets an auto-compound value for a delegation
    /// @custom:selector faa1786f
    /// @param candidate The address of the supported collator candidate
    /// @param value The percent of reward that should be auto-compounded
    /// @param candidateAutoCompoundingDelegationCount The number of auto-compounding delegations
    /// in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function setAutoCompound(
        bytes32 candidate,
        uint8 value,
        uint32 candidateAutoCompoundingDelegationCount,
        uint32 delegatorDelegationCount
    ) external;

    /// @dev Fetch the total staked amount of a delegator, regardless of the
    /// candidate.
    /// @custom:selector e6861713
    /// @param delegator Address of the delegator.
    /// @return Total amount of stake.
    function getDelegatorTotalStaked(bytes memory delegator)
    external
    view
    returns (uint256);

    /// @dev Fetch the total staked towards a candidate.
    /// @custom:selector bc5a1043
    /// @param candidate Address of the candidate.
    /// @return Total amount of stake.
    function getCandidateTotalCounted(bytes32 candidate)
    external
    view
    returns (uint256);
}
