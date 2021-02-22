import React, { useCallback } from "react";
import { Button, Card} from "antd";
import { Link } from "react-router-dom";
import { useConnection } from "../../contexts/connection";
import { useWallet } from "../../contexts/wallet";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { notify } from "../../utils/notifications";
import { ConnectButton } from "./../../components/ConnectButton";
import { LABELS } from "../../constants";
import {Line} from 'react-chartjs-2';

const solanaLogo = require ("../../img/solanaLogo.svg")

export const DiscoverView = () => {
  const connection = useConnection();
  const { publicKey } = useWallet();

  const state = {
    labels: ['January', 'February', 'March',
             'April', 'May'],
    datasets: [
      {
        label: 'Price',
        backgroundColor: 'rgba(75,192,192,1)',
        borderColor: 'rgba(0,0,0,1)',
        borderWidth: 2,
        data: [65, 59, 80, 81, 56]
      }
    ]
  }


  const airdrop = useCallback(() => {
    if (!publicKey) {
      return;
    }

    connection.requestAirdrop(publicKey, 2 * LAMPORTS_PER_SOL).then(() => {
      notify({
        message: LABELS.ACCOUNT_FUNDED,
        type: "success",
      });
    });
  }, [publicKey, connection]);

  return (
    <div style={{display:"flex", width:"100%", justifyContent: "center"}}>
      <div style={{width:"60%", marginRight:"20px", textAlign:"left", display:"flex", flexDirection:"column", justifyContent: "space-between"}}>
          <div>
            <h1>tSOL Share Price</h1>
            <Card className="card">
              <Line
                data={state}
                options={{
                  legend: {
                    display: false
                  }
                }}
              />
            </Card>
          </div>
          <div>
            <h1>tSOL Vault Rewards</h1>
            <Card className="card">
              <Line
            data={state}
              />
            </Card>
          </div>
          <div>
            <h1>About SOL</h1>
            <Card className="card">
              <Line
              data={state}
              />
            </Card>
          </div>
      </div>
      
      <div className = "card" style={{height: "450px", width:"30%", position: "sticky", top:0}}>
      <Card className="card" style={{ width: "100%", height: "100%"}}>
            <div style={{height: "100%",display: "flex", flexDirection:"column", justifyContent:"space-between"}}>
              <div>
                <Button>Deposit</Button>
                <Button>Withdraw</Button>
              </div>
              <div>
                <img src={solanaLogo} alt="solana logo" style={{maxWidth:"30%"}}/>
                <h2 className="subtext" style={{marginTop:"5px"}}>SOLANA</h2>
              </div>
              <div style={{marginTop: "5px", marginBottom:"5px"}}>
                <h2 className="title" style={{marginBottom: 0}}>15.5%</h2>
                <h3 style={{marginBottom: 0}}>Vault Rewards</h3>
                <h5>(projected APY)</h5>
              </div>
              <Link to="/discover">
                <Button className="tenderButton tenderButtonShade">Discover</Button>
              </Link>
            </div>
        </Card>
      </div>
    </div>
  );
};
