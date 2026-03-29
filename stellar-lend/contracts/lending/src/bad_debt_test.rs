#[cfg(test)]
mod test {
    use crate::{LendingContract, LendingContractClient};
    use soroban_sdk::{testutils::{Address as _}, Address, Env};

    #[test]
    fn test_bad_debt_accounting_auto_offset() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let liquidator = Address::generate(&env);
        let asset = Address::generate(&env);
        let collateral_asset = Address::generate(&env);

        let client = LendingContractClient::new(&env, &env.register_contract(None, LendingContract));

        // Initialize protocol
        client.initialize(&admin, &1000_000, &10);

        // 1. Credit Insurance Fund with 50 units
        client.credit_insurance_fund(&admin, &asset, &50);
        assert_eq!(client.get_insurance_fund_balance(&asset), 50);

        // 2. Setup a borrow position for the borrower
        // We bypass the collateral ratio check by depositing enough first, then borrowing.
        // Or in this simplified test, we just set the positions.
        client.deposit(&borrower, &collateral_asset, &80); 
        client.borrow(&borrower, &asset, &100, &collateral_asset, &0);

        // Verify initial state
        assert_eq!(client.get_total_bad_debt(&asset), 0);

        // 3. Liquidate: Repay 100 debt, but only 80 collateral exists.
        // Shortfall = 20.
        client.liquidate(&liquidator, &borrower, &asset, &collateral_asset, &100);

        // 4. Verify Accounting
        // Shortfall of 20 should have been created.
        // Since Insurance Fund had 50, it should auto-offset the 20.
        // Final Bad Debt: 0. Final Fund Balance: 50 - 20 = 30.
        assert_eq!(client.get_total_bad_debt(&asset), 0);
        assert_eq!(client.get_insurance_fund_balance(&asset), 30);
    }

    #[test]
    fn test_bad_debt_exceeding_insurance_fund() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let liquidator = Address::generate(&env);
        let asset = Address::generate(&env);
        let collateral_asset = Address::generate(&env);

        let client = LendingContractClient::new(&env, &env.register_contract(None, LendingContract));

        // Initialize protocol
        client.initialize(&admin, &1000_000, &10);

        // 1. Credit Insurance Fund with 10 units
        client.credit_insurance_fund(&admin, &asset, &10);

        // 2. Setup insolvent position: Debt 100, Collateral 70 -> Shortfall 30
        client.deposit(&borrower, &collateral_asset, &70);
        client.borrow(&borrower, &asset, &100, &collateral_asset, &0);

        // 3. Liquidate
        client.liquidate(&liquidator, &borrower, &asset, &collateral_asset, &100);

        // 4. Verify Accounting
        // Total shortfall: 30. 
        // Offset from fund: 10.
        // Remaining Bad Debt: 20. Fund Balance: 0.
        assert_eq!(client.get_total_bad_debt(&asset), 20);
        assert_eq!(client.get_insurance_fund_balance(&asset), 0);

        // 5. Manual offset: Credit fund with 50 and offset the remaining 20 bad debt
        client.credit_insurance_fund(&admin, &asset, &50);
        assert_eq!(client.get_insurance_fund_balance(&asset), 50);
        
        client.offset_bad_debt(&admin, &asset, &20);
        
        assert_eq!(client.get_total_bad_debt(&asset), 0);
        assert_eq!(client.get_insurance_fund_balance(&asset), 30);
    }
}
