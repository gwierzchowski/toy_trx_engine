using Random
using Printf

const ttypes = ["deposit", "withdrawal", "dispute", "resolve", "chargeback"];

function gen_tran(out_file::AbstractString, size::Integer)
    fin_trx = Set{Tuple{Int, Int}}();
    dis_trx = Set{Tuple{Int, Int}}();

    full_trx_range = 1:length(ttypes);
    client_range = 1:size ÷ 5;
    amount_max = 100.0;

    open(out_file, "w") do output
        println(output, "type,client,tx,amount");
        for trx in 1:size
            if trx < 3
                trx_range = 1:1;
            elseif trx < 10
                trx_range = 1:2;
            else
                trx_range = full_trx_range;
            end
            ttyp = rand(trx_range);
            if ttyp == 1
                cli = rand(client_range);
                amt = rand() * amount_max;
                push!(fin_trx, (cli, trx));
                println(output, ttypes[ttyp] * "," * string(cli) * "," * string(trx) * "," * @sprintf "%.4f" amt);
            elseif ttyp == 2
                cli, _ = rand(fin_trx);
                amt = rand() * amount_max;
                push!(fin_trx, (cli, trx));
                println(output, ttypes[ttyp] * "," * string(cli) * "," * string(trx) * "," * @sprintf "%.4f" amt);
            elseif ttyp == 3
                cli, ref = rand(fin_trx);
                push!(dis_trx, (cli, ref));
                println(output, ttypes[ttyp] * "," * string(cli) * "," * string(ref));
            elseif length(dis_trx) > 0
                cli, ref = rand(dis_trx);
                delete!(dis_trx, (cli, ref));
                println(output, ttypes[ttyp] * "," * string(cli) * "," * string(ref));
            else
                trx = trx - 1
            end
        end
    end
end

gen_tran("tests/samples/gen_100000.csv", 100000)